//! Driver processes: spawn, JSON-lines request/response, timeout, restart.

use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct DriverConfig {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub cwd: String,
    #[serde(default)]
    pub build: Option<String>,
    pub run: String,
    #[serde(default = "yes")]
    pub enabled: bool,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "driver")]
    pub drivers: Vec<DriverConfig>,
}

pub fn load_config(path: &Path) -> Result<Config, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Debug, Clone)]
pub enum Reply {
    Ok(Value),
    Err { code: String, message: String },
    /// The driver crashed, timed out or spoke something other than the protocol.
    Broken(String),
}

impl Reply {
    pub fn describe(&self) -> String {
        match self {
            Reply::Ok(_) => "ok".into(),
            Reply::Err { code, message } => format!("[{code}] {message}"),
            Reply::Broken(m) => format!("driver failure: {m}"),
        }
    }
}

pub struct Driver {
    pub cfg: DriverConfig,
    pub root: PathBuf,
    pub log_dir: PathBuf,
    pub timeout: Duration,
    pub hello: Value,
    pub caps: BTreeSet<String>,
    pub restarts: u32,
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    rx: Option<Receiver<Option<String>>>,
    next_id: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Pack,
    Open,
}

impl Driver {
    pub fn cwd(&self) -> PathBuf {
        self.root.join(&self.cfg.cwd)
    }

    /// Start the process and exchange `hello`.
    pub fn start(
        cfg: DriverConfig,
        root: &Path,
        log_dir: &Path,
        timeout: Duration,
    ) -> Result<Driver, String> {
        let mut d = Driver {
            cfg,
            root: root.to_path_buf(),
            log_dir: log_dir.to_path_buf(),
            timeout,
            hello: Value::Null,
            caps: BTreeSet::new(),
            restarts: 0,
            child: None,
            stdin: None,
            rx: None,
            next_id: 0,
        };
        d.spawn()?;
        Ok(d)
    }

    fn spawn(&mut self) -> Result<(), String> {
        let cwd = self.cwd();
        std::fs::create_dir_all(&self.log_dir).ok();
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_dir.join(format!("{}.stderr.log", self.cfg.name)))
            .map_err(|e| format!("stderr log: {e}"))?;
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!("exec {}", self.cfg.run))
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(log))
            .spawn()
            .map_err(|e| format!("spawn `{}` in {}: {e}", self.cfg.run, cwd.display()))?;
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::with_capacity(1 << 20, stdout);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(None);
                        break;
                    }
                    Ok(_) => {
                        if tx.send(Some(line)).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        self.stdin = child.stdin.take();
        self.child = Some(child);
        self.rx = Some(rx);
        match self.raw_call("hello", json!({})) {
            Reply::Ok(v) => {
                self.caps = v["capabilities"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|c| c.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                self.hello = v;
                Ok(())
            }
            other => {
                self.kill();
                Err(format!("hello failed: {}", other.describe()))
            }
        }
    }

    fn kill(&mut self) {
        self.stdin = None;
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.rx = None;
    }

    fn raw_call(&mut self, op: &str, params: Value) -> Reply {
        self.next_id += 1;
        let id = self.next_id;
        let mut req = params;
        if !req.is_object() {
            req = json!({});
        }
        req["id"] = json!(id);
        req["op"] = json!(op);
        let (Some(stdin), Some(rx)) = (self.stdin.as_mut(), self.rx.as_ref()) else {
            return Reply::Broken("driver not running".into());
        };
        let line = format!("{req}\n");
        if let Err(e) = stdin.write_all(line.as_bytes()).and_then(|_| stdin.flush()) {
            return Reply::Broken(format!("write to driver: {e}"));
        }
        let got = match rx.recv_timeout(self.timeout) {
            Ok(Some(l)) => l,
            Ok(None) | Err(RecvTimeoutError::Disconnected) => {
                return Reply::Broken(format!("driver exited during `{op}`"));
            }
            Err(RecvTimeoutError::Timeout) => {
                return Reply::Broken(format!("timeout after {:?} during `{op}`", self.timeout));
            }
        };
        let v: Value = match serde_json::from_str(&got) {
            Ok(v) => v,
            Err(e) => {
                let preview: String = got.chars().take(120).collect();
                return Reply::Broken(format!("non-JSON line from driver ({e}): {preview}"));
            }
        };
        if v["id"] != json!(id) {
            return Reply::Broken(format!("response id {} does not match request {id}", v["id"]));
        }
        match v["ok"].as_bool() {
            Some(true) => Reply::Ok(v["result"].clone()),
            Some(false) => Reply::Err {
                code: v["error"]["code"].as_str().unwrap_or("?").to_string(),
                message: v["error"]["message"].as_str().unwrap_or("").to_string(),
            },
            None => Reply::Broken(format!("response without ok: {}", crate::model::short(&v))),
        }
    }

    /// Send a request, restarting the driver first if an earlier one broke it.
    pub fn call(&mut self, op: &str, params: Value) -> Reply {
        if self.child.is_none() {
            self.restarts += 1;
            if let Err(e) = self.spawn() {
                return Reply::Broken(format!("restart failed: {e}"));
            }
        }
        let r = self.raw_call(op, params);
        if matches!(r, Reply::Broken(_)) {
            self.kill();
        }
        r
    }

    pub fn has(&self, cap: &str, dir: Dir) -> bool {
        let suffix = match dir {
            Dir::Pack => "pack",
            Dir::Open => "open",
        };
        self.caps.contains(cap) || self.caps.contains(&format!("{cap}:{suffix}"))
    }

    pub fn name(&self) -> &str {
        &self.cfg.name
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        self.kill();
    }
}
