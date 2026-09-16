//! JSON and Markdown reports.

use crate::result::{CaseResult, Status};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriverEntry {
    pub name: String,
    pub description: String,
    /// `ran` or `skipped`.
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub reason: String,
    pub hello: Value,
    pub capabilities: Vec<String>,
    pub restarts: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub tool: String,
    pub generated_at: String,
    pub seed: u64,
    pub spec: Value,
    pub suites: Vec<String>,
    pub drivers: Vec<DriverEntry>,
    pub summary: BTreeMap<String, Counts>,
    pub findings: Vec<Finding>,
    pub results: Vec<CaseResult>,
}

/// A known root cause from `findings.toml`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Finding {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite: Option<String>,
    #[serde(rename = "match")]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub spec: String,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Deserialize)]
struct FindingsFile {
    #[serde(default)]
    finding: Vec<Finding>,
}

pub fn load_findings(path: &std::path::Path) -> Result<Vec<Finding>, String> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let f: FindingsFile = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(f.finding)
}

/// Attribute each failure to the first finding that matches it.
pub fn attribute(results: &mut [CaseResult], findings: &[Finding]) {
    let compiled: Vec<(&Finding, Vec<regex::Regex>)> = findings
        .iter()
        .map(|f| (f, f.patterns.iter().filter_map(|p| regex::Regex::new(p).ok()).collect()))
        .collect();
    for r in results.iter_mut().filter(|r| matches!(r.status, Status::Fail | Status::Error)) {
        let hay = format!("{}\n{}\n{}", r.cause.as_deref().unwrap_or(""), r.detail, r.case);
        r.finding = compiled
            .iter()
            .find(|(f, res)| {
                f.suite.as_ref().is_none_or(|s| s == &r.suite)
                    && f.implementation.as_ref().is_none_or(|i| i == &r.receiver || i == &r.sender)
                    && res.iter().any(|re| re.is_match(&hay))
            })
            .map(|(f, _)| f.id.clone());
    }
}

#[derive(Serialize, Default, Clone, Copy)]
pub struct Counts {
    pub pass: usize,
    pub fail: usize,
    pub skip: usize,
    pub error: usize,
}

impl Counts {
    fn add(&mut self, s: Status) {
        match s {
            Status::Pass => self.pass += 1,
            Status::Fail => self.fail += 1,
            Status::Skip => self.skip += 1,
            Status::Error => self.error += 1,
        }
    }
    fn cell(&self) -> String {
        let ran = self.pass + self.fail + self.error;
        if ran == 0 {
            if self.skip > 0 { "–".into() } else { " ".into() }
        } else if self.fail + self.error == 0 {
            format!("✓ {}/{ran}", self.pass)
        } else {
            format!("✗ {}/{ran}", self.pass)
        }
    }
}

pub fn summarize(results: &[CaseResult]) -> BTreeMap<String, Counts> {
    let mut m: BTreeMap<String, Counts> = BTreeMap::new();
    for r in results {
        m.entry(r.suite.clone()).or_default().add(r.status);
    }
    m
}

/// Every capability the protocol defines, in display order.
const CAPABILITIES: &[&str] = &[
    "hpke-base", "signed-only", "sealed-box", "hpke-pq",
    "payload.scs", "payload.ctl", "payload.pad", "payload.rfi", "payload.rfa", "payload.rfd",
    "rfi.reply-path", "rfi.referral", "payload.hop", "padding", "payload-sender",
    "deterministic", "peek", "endpoint",
];

fn cap_cell(caps: &[String], cap: &str) -> &'static str {
    let has = |s: &str| caps.iter().any(|c| c == s);
    match (has(cap), has(&format!("{cap}:pack")), has(&format!("{cap}:open"))) {
        (true, _, _) | (_, true, true) => "✓",
        (_, true, false) => "pack only",
        (_, false, true) => "open only",
        _ => "—",
    }
}

fn esc(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', "<br>")
}

pub fn markdown(rep: &Report) -> String {
    let mut md = String::new();
    let ran: Vec<&DriverEntry> = rep.drivers.iter().filter(|d| d.status == "ran").collect();
    let names: Vec<String> = ran.iter().map(|d| d.name.clone()).collect();

    let _ = writeln!(md, "# TSP Rev 3 conformance report\n");
    let _ = writeln!(
        md,
        "Target **{}** at commit `{}` (`YTSP-AAC`). Generated {} with key seed `{}`.\n",
        rep.spec["spec"].as_str().unwrap_or("?"),
        rep.spec["commit"].as_str().unwrap_or("?"),
        rep.generated_at,
        rep.seed
    );
    let _ = writeln!(
        md,
        "Each case is **pass**, **fail** (with the diff), **skip** (a capability the implementation does not offer) or **error** (the harness or a driver broke). Cells read `✓ passed/ran` when nothing failed, `✗ passed/ran` when something did, `–` when every case skipped.\n"
    );

    // ---- implementations
    let _ = writeln!(md, "## Implementations\n");
    let _ = writeln!(md, "| Driver | Library | Version | Language | Status |");
    let _ = writeln!(md, "|---|---|---|---|---|");
    for d in &rep.drivers {
        let status = if d.status == "ran" {
            if d.restarts > 0 { format!("ran ({} restarts)", d.restarts) } else { "ran".into() }
        } else {
            format!("**skipped** — {}", esc(&d.reason))
        };
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} |",
            d.name,
            d.hello["name"].as_str().unwrap_or("—"),
            d.hello["version"].as_str().unwrap_or("—"),
            d.hello["language"].as_str().unwrap_or("—"),
            status
        );
    }

    // ---- summary
    let _ = writeln!(md, "\n## Summary\n");
    let _ = writeln!(md, "| Suite | Pass | Fail | Error | Skip |");
    let _ = writeln!(md, "|---|---:|---:|---:|---:|");
    for (s, c) in &rep.summary {
        let _ = writeln!(md, "| {s} | {} | {} | {} | {} |", c.pass, c.fail, c.error, c.skip);
    }

    // ---- capabilities
    let _ = writeln!(md, "\n## Capability matrix\n");
    let _ = writeln!(md, "As each driver declares in `hello`. `pack only`/`open only` mark one-sided support.\n");
    let _ = writeln!(md, "| Capability | {} |", names.join(" | "));
    let _ = writeln!(md, "|---|{}|", names.iter().map(|_| "---").collect::<Vec<_>>().join("|"));
    for cap in CAPABILITIES {
        let cells: Vec<&str> = ran.iter().map(|d| cap_cell(&d.capabilities, cap)).collect();
        let _ = writeln!(md, "| `{cap}` | {} |", cells.join(" | "));
    }
    // fields not reported on open
    let mut unrep: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in &rep.results {
        for u in &r.unreported {
            unrep.entry(r.receiver.clone()).or_default().insert(u.clone());
        }
    }
    let cells: Vec<String> = names
        .iter()
        .map(|n| {
            unrep
                .get(n)
                .map(|s| s.iter().map(|x| format!("`{x}`")).collect::<Vec<_>>().join(", "))
                .unwrap_or_else(|| "—".into())
        })
        .collect();
    let _ = writeln!(md, "| *not reported by `open`* | {} |", cells.join(" | "));

    // ---- matrices
    for suite in &rep.suites {
        let rs: Vec<&CaseResult> = rep.results.iter().filter(|r| &r.suite == suite).collect();
        if rs.is_empty() {
            continue;
        }
        match suite.as_str() {
            "interop" | "relationship" => {
                let (row, col) = if suite == "interop" { ("packs ↓ / opens →", "") } else { ("A ↓ / B →", "") };
                let _ = writeln!(md, "\n## {} matrix\n", title(suite));
                let _ = write!(md, "| {row}{col} |");
                for n in &names {
                    let _ = write!(md, " {n} |");
                }
                let _ = writeln!(md, "\n|---|{}|", names.iter().map(|_| "---").collect::<Vec<_>>().join("|"));
                for s in &names {
                    let _ = write!(md, "| **{s}** |");
                    for r in &names {
                        let mut c = Counts::default();
                        rs.iter().filter(|x| &x.sender == s && &x.receiver == r).for_each(|x| c.add(x.status));
                        let _ = write!(md, " {} |", c.cell());
                    }
                    let _ = writeln!(md);
                }
                // per case
                let pairs: Vec<(String, String)> = names
                    .iter()
                    .flat_map(|s| names.iter().map(move |r| (s.clone(), r.clone())))
                    .collect();
                let mut cases: Vec<String> = vec![];
                for r in &rs {
                    if !cases.contains(&r.case) {
                        cases.push(r.case.clone());
                    }
                }
                let _ = writeln!(md, "\n<details><summary>Per case ({} cases × {} pairs)</summary>\n", cases.len(), pairs.len());
                let _ = write!(md, "| Case |");
                for (s, r) in &pairs {
                    let _ = write!(md, " {}→{} |", abbrev(s), abbrev(r));
                }
                let _ = writeln!(md, "\n|---|{}|", pairs.iter().map(|_| ":-:").collect::<Vec<_>>().join("|"));
                for case in &cases {
                    let _ = write!(md, "| `{case}` |");
                    for (s, r) in &pairs {
                        let st = rs.iter().find(|x| &x.case == case && &x.sender == s && &x.receiver == r).map(|x| x.status);
                        let _ = write!(md, " {} |", mark(st));
                    }
                    let _ = writeln!(md);
                }
                let _ = writeln!(md, "\n</details>");
            }
            _ => {
                let _ = writeln!(md, "\n## {} matrix\n", title(suite));
                let (rows, key): (Vec<String>, fn(&CaseResult) -> String) = if suite == "negative" {
                    (
                        ordered(rs.iter().map(|r| mutation_of(r))),
                        |r: &CaseResult| mutation_of(r),
                    )
                } else {
                    (ordered(rs.iter().map(|r| r.case.clone())), |r: &CaseResult| r.case.clone())
                };
                if suite == "negative" {
                    let _ = writeln!(md, "Aggregated over the message sources (the implementation's own output and the Appendix A vectors); see the JSON report for each source.\n");
                }
                let _ = writeln!(md, "| Case | {} |", names.join(" | "));
                let _ = writeln!(md, "|---|{}|", names.iter().map(|_| ":-:").collect::<Vec<_>>().join("|"));
                for row in rows {
                    let _ = write!(md, "| `{row}` |");
                    for n in &names {
                        let mut c = Counts::default();
                        rs.iter().filter(|x| &x.receiver == n && key(x) == row).for_each(|x| c.add(x.status));
                        let cell = if suite == "negative" { c.cell() } else {
                            mark(rs.iter().find(|x| &x.receiver == n && key(x) == row).map(|x| x.status)).to_string()
                        };
                        let _ = write!(md, " {cell} |");
                    }
                    let _ = writeln!(md);
                }
            }
        }
    }

    // ---- findings
    let _ = writeln!(md, "\n## Findings\n");
    let failing: Vec<&CaseResult> = rep.results.iter().filter(|r| matches!(r.status, Status::Fail | Status::Error)).collect();
    if failing.is_empty() {
        let _ = writeln!(md, "No failures.");
    }
    let mut n = 0;
    let known: Vec<&Finding> = rep.findings.iter().filter(|f| failing.iter().any(|r| r.finding.as_deref() == Some(&f.id))).collect();
    if !known.is_empty() {
        let _ = writeln!(md, "Failures attributed to an investigated root cause (`findings.toml`). *Kind*: `spec-violation` contradicts a MUST; `disagreement` — implementations differ where the specification is silent or ambiguous; `limit` — a bound the specification does not set; `api-gap` — the library cannot express what the protocol needs.\n");
        let _ = writeln!(md, "| # | Finding | Kind | Cases |\n|---|---|---|---:|");
        for (i, f) in known.iter().enumerate() {
            let c = failing.iter().filter(|r| r.finding.as_deref() == Some(&f.id)).count();
            let _ = writeln!(md, "| {} | {} | `{}` | {c} |", i + 1, esc(&f.title), f.kind);
        }
        let _ = writeln!(md);
    }
    for f in &known {
        n += 1;
        let rs: Vec<&&CaseResult> = failing.iter().filter(|r| r.finding.as_deref() == Some(&f.id)).collect();
        let _ = writeln!(md, "### {n}. {}\n", esc(&f.title));
        let _ = writeln!(md, "**Kind:** `{}` · **Spec:** {}\n", f.kind, f.spec);
        let _ = writeln!(md, "{}\n", f.explanation.trim());
        let _ = writeln!(md, "Observed in {} case(s); for example `{}/{}` ({} → {}):\n", rs.len(), rs[0].suite, rs[0].case, rs[0].sender, rs[0].receiver);
        let _ = writeln!(md, "```text\n{}\n```\n", rs[0].detail.chars().take(800).collect::<String>());
        let affected: Vec<String> = rs.iter().map(|r| format!("- `{}/{}` {} → {}", r.suite, r.case, r.sender, r.receiver)).collect();
        let _ = writeln!(md, "<details><summary>Affected cases</summary>\n\n{}\n\n</details>\n", affected.join("\n"));
    }
    let mut groups: BTreeMap<String, Vec<&CaseResult>> = BTreeMap::new();
    for r in failing.iter().filter(|r| r.finding.is_none()) {
        groups.entry(r.cause.clone().unwrap_or_else(|| "unclassified".into())).or_default().push(r);
    }
    if !groups.is_empty() && !known.is_empty() {
        let _ = writeln!(md, "### Not yet investigated\n\nGrouped by the runner's own cause string.\n");
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (cause, rs) in groups.iter() {
        n += 1;
        let _ = writeln!(md, "### {}. {}\n", n, esc(cause));
        let _ = writeln!(md, "{} case(s). Example — `{}/{}` ({} → {}), spec {}:\n", rs.len(), rs[0].suite, rs[0].case, rs[0].sender, rs[0].receiver, rs[0].spec);
        let _ = writeln!(md, "```text\n{}\n```\n", rs[0].detail.chars().take(1200).collect::<String>());
        let mut affected: Vec<String> = rs.iter().map(|r| format!("`{}/{}` {}→{}", r.suite, r.case, r.sender, r.receiver)).collect();
        affected.dedup();
        if affected.len() > 1 {
            let _ = writeln!(md, "<details><summary>All affected cases</summary>\n\n{}\n\n</details>\n", affected.iter().map(|a| format!("- {a}")).collect::<Vec<_>>().join("\n"));
        }
    }

    // ---- warnings
    let warned: Vec<&CaseResult> = rep.results.iter().filter(|r| !r.warnings.is_empty()).collect();
    if !warned.is_empty() {
        let _ = writeln!(md, "## Warnings\n");
        let _ = writeln!(md, "Correct rejections under an error code other than the one expected (the specification mandates no taxonomy).\n");
        let mut w: BTreeMap<(String, String), usize> = BTreeMap::new();
        for r in warned {
            for x in &r.warnings {
                let code = x.split(']').next().unwrap_or("").trim_start_matches("refused with [").to_string();
                *w.entry((r.receiver.clone(), format!("{} → [{code}]", mutation_of(r)))).or_default() += 1;
            }
        }
        let _ = writeln!(md, "| Implementation | Case → code | Count |\n|---|---|---:|");
        for ((i, c), n) in w {
            let _ = writeln!(md, "| {i} | `{c}` | {n} |");
        }
        let _ = writeln!(md);
    }

    // ---- skips
    let _ = writeln!(md, "## Skip reasons\n");
    let mut skips: BTreeMap<String, usize> = BTreeMap::new();
    for r in rep.results.iter().filter(|r| r.status == Status::Skip) {
        *skips.entry(crate::result::normalize(&r.detail)).or_default() += 1;
    }
    let mut skips: Vec<_> = skips.into_iter().collect();
    skips.sort_by(|a, b| b.1.cmp(&a.1));
    let _ = writeln!(md, "| Reason | Cases |\n|---|---:|");
    for (reason, n) in skips {
        let _ = writeln!(md, "| {} | {n} |", esc(&reason));
    }
    md
}

fn mutation_of(r: &CaseResult) -> String {
    r.case.rsplit('/').next().unwrap_or(&r.case).to_string()
}

fn ordered(it: impl Iterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    for x in it {
        if !v.contains(&x) {
            v.push(x);
        }
    }
    v
}

fn title(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn abbrev(s: &str) -> String {
    s.split('-').next().unwrap_or(s).chars().take(5).collect()
}

fn mark(s: Option<Status>) -> &'static str {
    match s {
        Some(Status::Pass) => "✓",
        Some(Status::Fail) => "✗",
        Some(Status::Skip) => "–",
        Some(Status::Error) => "E",
        None => " ",
    }
}
