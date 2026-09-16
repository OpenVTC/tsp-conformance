//! `tsp-conformance` — cross-implementation conformance runner for TSP Rev 3.

mod cesr;
mod compare;
mod ctx;
mod driver;
mod model;
mod report;
mod resign;
mod result;
mod suites;

use clap::Parser;
use ctx::Ctx;
use driver::{Driver, load_config};
use model::KeyFactory;
use report::{DriverEntry, Report};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const ALL_SUITES: &[&str] = &["vectors", "interop", "negative", "relationship"];

#[derive(Parser, Debug)]
#[command(name = "tsp-conformance", about = "Cross-implementation conformance runner for TSP Rev 3")]
struct Args {
    /// Driver configuration.
    #[arg(long, default_value = "drivers.toml")]
    config: PathBuf,
    /// Drivers to run (comma separated). Default: every enabled driver.
    #[arg(long, value_delimiter = ',')]
    drivers: Vec<String>,
    /// Suites to run (comma separated): vectors, interop, negative, relationship.
    #[arg(long, value_delimiter = ',')]
    suites: Vec<String>,
    /// Only cases whose `suite/case` contains one of these substrings.
    #[arg(long = "case")]
    cases: Vec<String>,
    /// Run each selected driver's build command first.
    #[arg(long)]
    build: bool,
    /// Seed for generated keys (default: random, printed in the report).
    #[arg(long)]
    seed: Option<u64>,
    /// Per-request timeout in seconds.
    #[arg(long, default_value_t = 60)]
    timeout: u64,
    /// Appendix A fixture.
    #[arg(long, default_value = "fixtures/spec-vectors.json")]
    fixture: PathBuf,
    /// JSON report path.
    #[arg(long, default_value = "reports/report.json")]
    json: PathBuf,
    /// Markdown report path.
    #[arg(long, default_value = "reports/report.md")]
    markdown: PathBuf,
    /// Print every case, not just failures.
    #[arg(short, long)]
    verbose: bool,
    /// Rebuild the Markdown from an existing JSON report and exit.
    #[arg(long)]
    from_json: Option<PathBuf>,
    /// Build (implies --build) and start each selected driver, report, and exit.
    #[arg(long)]
    build_only: bool,
    /// A note printed under the report title (e.g. which branches were built).
    #[arg(long, default_value = "")]
    note: String,
    /// Known root causes used to group failures in the report.
    #[arg(long, default_value = "findings.toml")]
    findings: PathBuf,
    /// What makes the exit status non-zero. `all`: any failing case. `known`:
    /// only a failure no finding in `findings.toml` explains, or a driver that
    /// did not run — the CI gate, which catches regressions and new
    /// disagreements without going red on already-investigated ones.
    #[arg(long, value_enum, default_value_t = Gate::All)]
    gate: Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Gate {
    All,
    Known,
}

fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let (h, m, s) = (secs % 86_400 / 3600, secs % 3600 / 60, secs % 60);
    // civil-from-days (Howard Hinnant)
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn build(cfg: &driver::DriverConfig, cwd: &Path, logs: &Path) -> Result<(), String> {
    let Some(cmd) = &cfg.build else { return Ok(()) };
    eprintln!("building {}: {cmd}", cfg.name);
    std::fs::create_dir_all(logs).ok();
    let log_path = logs.join(format!("{}.build.log", cfg.name));
    let out = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .envs(&cfg.env)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("cannot run build: {e}"))?;
    let mut log = out.stdout.clone();
    log.extend_from_slice(&out.stderr);
    std::fs::write(&log_path, &log).ok();
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("build failed ({}); see {}", out.status, log_path.display()))
    }
}

fn main() {
    let args = Args::parse();

    if let Some(path) = &args.from_json {
        let text = std::fs::read_to_string(path).expect("read JSON report");
        let rep: serde_json::Value = serde_json::from_str(&text).expect("parse JSON report");
        let mut rep: Report = report_from_value(rep);
        rep.findings = report::load_findings(&args.findings).unwrap_or_default();
        report::attribute(&mut rep.results, &rep.findings);
        write_out(&args.markdown, &report::markdown(&rep));
        return;
    }

    let cfg = load_config(&args.config).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(2);
    });
    let root = args
        .config
        .parent()
        .map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p })
        .unwrap_or(Path::new("."))
        .to_path_buf();
    let logs = args.json.parent().unwrap_or(Path::new(".")).join("logs");
    let suites: Vec<String> = if args.suites.is_empty() {
        ALL_SUITES.iter().map(|s| s.to_string()).collect()
    } else {
        args.suites.clone()
    };
    for s in &suites {
        if !ALL_SUITES.contains(&s.as_str()) {
            eprintln!("error: unknown suite {s}");
            std::process::exit(2);
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&args.fixture).expect("read fixture"),
    )
    .expect("parse fixture");

    let mut entries = vec![];
    let mut drivers = vec![];
    for dc in cfg.drivers {
        let chosen = if args.drivers.is_empty() {
            dc.enabled
        } else {
            args.drivers.contains(&dc.name)
        };
        if !chosen {
            continue;
        }
        let cwd = root.join(&dc.cwd);
        let skip = |reason: String, dc: &driver::DriverConfig| DriverEntry {
            name: dc.name.clone(),
            description: dc.description.clone(),
            overrides: dc.overridden.clone(),
            status: "skipped".into(),
            reason,
            hello: serde_json::Value::Null,
            capabilities: vec![],
            restarts: 0,
        };
        if !cwd.is_dir() {
            let reason = format!("directory {} does not exist", cwd.display());
            eprintln!("skipping {}: {reason}", dc.name);
            entries.push(skip(reason, &dc));
            continue;
        }
        if (args.build || args.build_only)
            && let Err(reason) = build(&dc, &cwd, &logs)
        {
            eprintln!("skipping {}: {reason}", dc.name);
            entries.push(skip(reason, &dc));
            continue;
        }
        match Driver::start(dc.clone(), &root, &logs, Duration::from_secs(args.timeout)) {
            Ok(d) => {
                eprintln!(
                    "started {} ({} {}), {} capabilities",
                    d.name(),
                    d.hello["name"].as_str().unwrap_or("?"),
                    d.hello["version"].as_str().unwrap_or("?"),
                    d.caps.len()
                );
                drivers.push(d);
            }
            Err(e) => {
                let reason = format!("failed to start: {e}");
                eprintln!("skipping {}: {reason}", dc.name);
                entries.push(skip(reason, &dc));
            }
        }
    }

    if args.build_only {
        for d in &drivers {
            eprintln!("ready    {} — {} capabilities: {}", d.name(), d.caps.len(), d.caps.iter().cloned().collect::<Vec<_>>().join(" "));
        }
        for e in &entries {
            eprintln!("skipped  {} — {}", e.name, e.reason);
        }
        return;
    }
    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    });
    eprintln!("key seed {seed}");
    let mut ctx = Ctx {
        drivers,
        fixture,
        keys: KeyFactory::new(seed),
        filters: args.cases.clone(),
        results: vec![],
        verbose: args.verbose,
    };
    let n = ctx.drivers.len();
    for suite in &suites {
        eprintln!("suite {suite}");
        match suite.as_str() {
            "vectors" => (0..n).for_each(|i| suites::vectors::run(&mut ctx, i)),
            "interop" => {
                for p in 0..n {
                    for r in 0..n {
                        suites::interop::run(&mut ctx, p, r);
                    }
                }
                for x in 0..n {
                    for p in 0..n {
                        for r in 0..n {
                            suites::interop::run_mixed(&mut ctx, x, p, r);
                        }
                    }
                }
            }
            "negative" => (0..n).for_each(|i| suites::negative::run(&mut ctx, i)),
            "relationship" => {
                for a in 0..n {
                    for b in 0..n {
                        suites::relationship::run(&mut ctx, a, b);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    let findings = report::load_findings(&args.findings).unwrap_or_else(|e| {
        eprintln!("warning: findings not loaded: {e}");
        vec![]
    });
    report::attribute(&mut ctx.results, &findings);
    let mut all_entries: Vec<DriverEntry> = ctx
        .drivers
        .iter()
        .map(|d| DriverEntry {
            name: d.name().into(),
            description: d.cfg.description.clone(),
            overrides: d.cfg.overridden.clone(),
            status: "ran".into(),
            reason: String::new(),
            hello: d.hello.clone(),
            capabilities: d.caps.iter().cloned().collect(),
            restarts: d.restarts,
        })
        .collect();
    all_entries.extend(entries);
    let rep = Report {
        tool: format!("tsp-conformance {}", env!("CARGO_PKG_VERSION")),
        generated_at: now_rfc3339(),
        note: args.note.clone(),
        seed,
        spec: ctx.fixture["_source"].clone(),
        suites: suites.clone(),
        summary: report::summarize(&ctx.results),
        drivers: all_entries,
        findings,
        results: std::mem::take(&mut ctx.results),
    };
    write_out(&args.json, &serde_json::to_string_pretty(&rep).unwrap());
    write_out(&args.markdown, &report::markdown(&rep));
    let failed = rep.summary.values().map(|c| c.fail + c.error).sum::<usize>();
    for (s, c) in &rep.summary {
        eprintln!("{s:13} pass {:4}  fail {:4}  error {:3}  skip {:4}", c.pass, c.fail, c.error, c.skip);
    }
    eprintln!("reports: {} {}", args.json.display(), args.markdown.display());
    let red = match args.gate {
        Gate::All => failed > 0,
        Gate::Known => {
            let fixed = |id: &str| rep.findings.iter().any(|f| f.id == id && f.fixed.is_some());
            let unexplained: Vec<_> = rep
                .results
                .iter()
                .filter(|r| matches!(r.status, result::Status::Fail | result::Status::Error))
                .filter(|r| r.finding.as_deref().is_none_or(fixed))
                .collect();
            for r in &unexplained {
                let why = match &r.finding {
                    Some(id) => format!("REGRESSION ({id})"),
                    None => "UNEXPLAINED".to_string(),
                };
                eprintln!("{why}  {} [{} -> {}] {}", r.case, r.sender, r.receiver, r.detail);
            }
            let not_run: Vec<_> = rep.drivers.iter().filter(|d| d.status != "ran").collect();
            for d in &not_run {
                eprintln!("DRIVER NOT RUN  {}: {}", d.name, d.reason);
            }
            let stale: Vec<_> = rep
                .findings
                .iter()
                .filter(|f| f.fixed.is_none() && !f.patterns.is_empty())
                .filter(|f| !rep.results.iter().any(|r| r.finding.as_deref() == Some(f.id.as_str())))
                .collect();
            for f in &stale {
                eprintln!("note: finding {} matched no failure this run (fixed?)", f.id);
            }
            eprintln!(
                "gate known: {} unexplained or regressed failure(s), {} driver(s) not run",
                unexplained.len(),
                not_run.len()
            );
            !unexplained.is_empty() || !not_run.is_empty()
        }
    };
    std::process::exit(if red { 1 } else { 0 });
}

fn write_out(path: &Path, text: &str) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).ok();
    }
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

/// Rehydrate a report written by this tool (for `--from-json`).
fn report_from_value(v: serde_json::Value) -> Report {
    use result::{CaseResult, Status};
    let status = |s: &str| match s {
        "pass" => Status::Pass,
        "fail" => Status::Fail,
        "skip" => Status::Skip,
        _ => Status::Error,
    };
    let strs = |x: &serde_json::Value| -> Vec<String> {
        x.as_array().map(|a| a.iter().filter_map(|s| s.as_str().map(str::to_string)).collect()).unwrap_or_default()
    };
    let results: Vec<CaseResult> = v["results"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| CaseResult {
            suite: r["suite"].as_str().unwrap_or_default().into(),
            case: r["case"].as_str().unwrap_or_default().into(),
            spec: r["spec"].as_str().unwrap_or_default().into(),
            sender: r["sender"].as_str().unwrap_or_default().into(),
            receiver: r["receiver"].as_str().unwrap_or_default().into(),
            status: status(r["status"].as_str().unwrap_or("")),
            detail: r["detail"].as_str().unwrap_or_default().into(),
            warnings: strs(&r["warnings"]),
            unreported: strs(&r["unreported"]),
            cause: r["cause"].as_str().map(str::to_string),
            finding: None,
        })
        .collect();
    let drivers = v["drivers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| DriverEntry {
            name: d["name"].as_str().unwrap_or_default().into(),
            description: d["description"].as_str().unwrap_or_default().into(),
            overrides: strs(&d["overrides"]),
            status: d["status"].as_str().unwrap_or_default().into(),
            reason: d["reason"].as_str().unwrap_or_default().into(),
            hello: d["hello"].clone(),
            capabilities: strs(&d["capabilities"]),
            restarts: d["restarts"].as_u64().unwrap_or(0) as u32,
        })
        .collect();
    Report {
        tool: v["tool"].as_str().unwrap_or_default().into(),
        generated_at: v["generatedAt"].as_str().unwrap_or_default().into(),
        note: v["note"].as_str().unwrap_or_default().into(),
        seed: v["seed"].as_u64().unwrap_or(0),
        spec: v["spec"].clone(),
        suites: strs(&v["suites"]),
        summary: report::summarize(&results),
        findings: vec![],
        drivers,
        results,
    }
}
