//! Daemon wire-shape goldens (contracts/DAEMON.md §4): every Request
//! and Response variant's canonical NDJSON is frozen under
//! contracts/fixtures/daemon/ — a serde rename, tag change or field
//! reorder reddens this suite BEFORE a second client (the M6 GUI)
//! can be desynced. CE_BLESS=1 regenerates deliberately. The tag
//! functions carry NO wildcard arm: adding a variant breaks compile
//! here, which is the walk to this file's fixture discipline.

use crate::common;
use codeeraser::daemon::proto::{Request, Response};
use serde_json::json;

fn requests() -> Vec<Request> {
    vec![
        // the pre-1.1.0 line: token defaults empty and is skipped on
        // serialize, so the old golden bytes still hold
        Request::Hello {
            proto: "1.0.0".into(),
            token: String::new(),
        },
        Request::Hello {
            proto: "1.1.0".into(),
            token: "a3f2".into(),
        },
        Request::Ping,
        Request::Dedup {
            min_tokens: Some(50),
            min_distinct: Some(7),
        },
        Request::Dedup {
            min_tokens: None,
            min_distinct: None,
        },
        Request::Probe {
            file_path: "src/a.rs".into(),
            content: "fn x() {}\n".into(),
        },
        Request::FourClass {
            pairs: vec![
                (Some("a.rs".into()), Some("a.rs".into())),
                (None, Some("new.rs".into())),
            ],
        },
        Request::Shutdown,
    ]
}

fn replies() -> Vec<Response> {
    vec![
        Response::HelloOk {
            proto: "1.0.0".into(),
        },
        Response::Restart {
            reason: "major skew: client 2.0.0 vs daemon 1.0.0".into(),
        },
        Response::Pong { uptime_ms: 12 },
        Response::DedupReport {
            report: json!({"blocks": [], "groups": []}),
        },
        Response::ProbeReport {
            matches: json!([]),
            elapsed_ms: 3,
        },
        Response::FourClassReport {
            report: json!({"pairs": [], "degraded": null}),
        },
        Response::Error {
            message: "bad request: expected value".into(),
        },
        Response::Bye,
    ]
}

fn req_tag(r: &Request) -> &'static str {
    match r {
        Request::Hello { .. } => "hello",
        Request::Ping => "ping",
        Request::Dedup { .. } => "dedup",
        Request::Probe { .. } => "probe",
        Request::FourClass { .. } => "four_class",
        Request::Shutdown => "shutdown",
    }
}

fn resp_tag(r: &Response) -> &'static str {
    match r {
        Response::HelloOk { .. } => "hello_ok",
        Response::Restart { .. } => "restart",
        Response::Pong { .. } => "pong",
        Response::DedupReport { .. } => "dedup_report",
        Response::ProbeReport { .. } => "probe_report",
        Response::FourClassReport { .. } => "four_class_report",
        Response::Error { .. } => "error",
        Response::Bye => "bye",
    }
}

/// Serialize the battery, pin the bytes, and prove the round trip:
/// parse-back must re-serialize to the identical line.
fn frozen_ndjson<T: serde::Serialize + serde::de::DeserializeOwned>(
    items: &[T],
    tags: &[&'static str],
) -> String {
    let mut lines = Vec::new();
    for (item, tag) in items.iter().zip(tags) {
        let line = serde_json::to_string(item).expect("serialize");
        assert!(
            line.contains(&format!("\"type\":\"{tag}\"")),
            "{tag}: tag drifted in {line}"
        );
        let back: T = serde_json::from_str(&line).expect("parse back");
        assert_eq!(
            serde_json::to_string(&back).expect("re-serialize"),
            line,
            "{tag}: round trip not identity"
        );
        lines.push(line);
    }
    lines.join("\n")
}

/// One throat for both directions (the ratchet caught the twin
/// per-direction tests on landing).
fn freeze<T: serde::Serialize + serde::de::DeserializeOwned>(
    items: Vec<T>,
    tag: fn(&T) -> &'static str,
    file: &str,
) {
    let tags: Vec<_> = items.iter().map(tag).collect();
    common::assert_matches_golden(&frozen_ndjson(&items, &tags), &common::golden_path(file));
}

#[test]
fn wire_shapes_are_frozen() {
    // count pins beside the wildcard-free matches: adding a variant
    // breaks req_tag/resp_tag at COMPILE time, and these two lines
    // are where the walk ends — bump each count WITH its fixture
    // line, or the "every variant is frozen" claim silently rots
    // (clearance review). requests() carries 7 items over 6 variants
    // (Dedup appears twice to pin the null-optional shape).
    assert_eq!(
        requests().len(),
        8,
        "request battery: 6 variants + the None-Dedup and tokenless-hello shapes"
    );
    assert_eq!(replies().len(), 8, "reply battery covers all 8 variants");
    freeze(requests(), req_tag, "daemon/requests.ndjson");
    freeze(replies(), resp_tag, "daemon/replies.ndjson");
}
