//! The doctor is ONE measurement with three faces (K round step 6).
//!
//! Before the document existed, `ce doctor` printed prose and nothing
//! else could ask the same question — so the GUI had no diagnostic at
//! all, and any second face would have been a second measurement that
//! agreed most days. These legs pin the two properties that make one
//! measurement worth having: the document says everything the console
//! line says, and a missing core is REPORTED rather than thrown.

mod common;

use codeeraser::health::doctor;

/// A tree with no `.ce` and no core: every field still answers, and
/// the handshake failure rides IN the document. A doctor that returns
/// Err tells the operator nothing about the machine they came to ask
/// about — the one situation they reached for it in.
#[test]
fn a_missing_core_is_a_finding_not_an_error() {
    let dir = common::tmp("doctor-nocore");
    let d = doctor::document(&dir, "definitely-not-a-real-core-binary");
    assert_eq!(d["schema"], doctor::SCHEMA_ID);
    assert_eq!(d["core"]["handshake"], serde_json::json!(false));
    assert!(
        d["core"]["error"].as_str().is_some_and(|e| !e.is_empty()),
        "the failure names itself: {:?}",
        d["core"]["error"]
    );
    assert_eq!(d["core"]["version"], serde_json::Value::Null);
    // the project facts answer anyway — they never needed the core
    assert!(d["root"].as_str().is_some_and(|r| !r.is_empty()));
    assert!(d["guard"].as_str().is_some());
    // a CODE since 0.2.0 (plan v2.15): 0 = absent. The prose that
    // used to sit here belongs to each face now, in its own
    // language, which is the whole point of the change.
    assert_eq!(d["index"]["state"], 0);
    assert_eq!(d["index"]["files"], serde_json::Value::Null);
    assert_eq!(d["degradedRuns"]["entries"], 0);
}

/// The document agrees with the console face field for field — the
/// console renders THIS object, so a drift here is the two-faces
/// defect the document was introduced to make impossible.
#[test]
fn the_console_line_is_built_from_the_document() {
    let dir = common::tmp("doctor-console");
    let core = common::core_bin();
    let d = doctor::document(&dir, &core);
    let out = common::run_ce(&dir, &["doctor", "--core", &core]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    // index and daemon carry CODES since 0.2.0 (plan v2.15), so a
    // text comparison against the document is meaningless — and a
    // stronger claim is available instead: the binary printed what
    // `console()` renders from THIS object, line for line. Both
    // sides are deterministic in a fresh tmp root (index absent, no
    // daemon running), so no timing rides in the comparison.
    let (want_lines, handshook) = doctor::console(&d);
    assert!(handshook, "this leg runs with a real core");
    for want in &want_lines {
        assert!(
            text.contains(want),
            "console omits {want:?}
{text}"
        );
    }
    assert!(text.contains(d["ce"]["version"].as_str().expect("version")));
    assert!(
        text.contains(&format!(
            "{} of {}",
            d["degradedRuns"]["degraded"], d["degradedRuns"]["entries"]
        )),
        "console omits the degraded frame\n{text}"
    );
}
