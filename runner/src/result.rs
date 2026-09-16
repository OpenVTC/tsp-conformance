//! Case outcomes.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseResult {
    pub suite: String,
    pub case: String,
    /// Spec section(s) the case traces to.
    pub spec: String,
    /// Packing side (interop), message source (negative), or `A` endpoint.
    pub sender: String,
    /// Opening side, or `B` endpoint. For per-implementation suites this is
    /// the implementation under test.
    pub receiver: String,
    pub status: Status,
    /// The diff for a failure, the missing capability for a skip.
    pub detail: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Fields an implementation did not report (not compared).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unreported: Vec<String>,
    /// Grouping key for the findings list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// The known finding (findings.toml) this failure is attributed to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finding: Option<String>,
}

/// What a case body returns; the suite fills in the identifying fields.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub status: Status,
    pub detail: String,
    pub warnings: Vec<String>,
    pub unreported: Vec<String>,
    pub cause: Option<String>,
}

impl Outcome {
    pub fn pass() -> Self {
        Outcome {
            status: Status::Pass,
            detail: String::new(),
            warnings: vec![],
            unreported: vec![],
            cause: None,
        }
    }
    pub fn skip(why: impl Into<String>) -> Self {
        Outcome {
            status: Status::Skip,
            detail: why.into(),
            ..Outcome::pass()
        }
    }
    pub fn fail(detail: impl Into<String>, cause: impl Into<String>) -> Self {
        Outcome {
            status: Status::Fail,
            detail: detail.into(),
            cause: Some(cause.into()),
            ..Outcome::pass()
        }
    }
    pub fn error(detail: impl Into<String>) -> Self {
        let d = detail.into();
        Outcome {
            status: Status::Error,
            cause: Some(format!("harness/driver error: {}", normalize(&d))),
            detail: d,
            ..Outcome::pass()
        }
    }
}

/// Strip run-specific noise (VIDs, base64 blobs, numbers) so messages that
/// differ only in data group together.
pub fn normalize(s: &str) -> String {
    let mut out = String::new();
    for word in s.split_whitespace() {
        let w = word.trim_matches(|c: char| c == ',' || c == '.' || c == ':' || c == '"');
        let noisy = w.starts_with("did:")
            || (w.len() > 20 && w.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            || w.chars().any(|c| c.is_ascii_digit()) && w.len() > 3;
        if noisy {
            out.push_str("… ");
        } else {
            out.push_str(word);
            out.push(' ');
        }
    }
    let t = out.trim().to_string();
    t.chars().take(160).collect()
}
