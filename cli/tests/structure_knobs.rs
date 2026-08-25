//! The split-ROI advisory prices with the knobs `ce.toml` declares —
//! specifically P_max, whose omission made two families disagree
//! about one curve (6.1.0).
//!
//! `verdict/1` has always received `score.size_penalty_max` through
//! ceiling code 3, while `structure/1` received only the soft line
//! (12) and the hard line (13). A repo that declared the knob got the
//! declared curve in its score and the core's built-in 10 in its
//! advisory — and because both halves were internally consistent,
//! nothing anywhere disagreed out loud. The leg below is the only
//! kind of evidence that catches that class: not "the knob is in the
//! request" but "the NUMBER the user reads moves when the knob does".

mod common;

use codeeraser::structure::judge;
use std::path::Path;

/// A file well past a low declared soft line, with two top-level
/// units so a seam exists to price at all.
const WIDE: &str = "def first(values):
    total = 0
    for item in values:
        total = total + item
    return total


def second(values):
    total = 0
    for item in values:
        total = total - item
    return total
";

fn fixture(name: &str, penalty_max: Option<u32>) -> std::path::PathBuf {
    let dir = common::tmp(name);
    let mut toml = String::from("[thresholds]\nfile_lines_warn = 5\nfile_lines_fail = 40\n");
    if let Some(p) = penalty_max {
        toml.push_str(&format!("\n[score]\nsize_penalty_max = {p}\n"));
    }
    std::fs::write(dir.join("ce.toml"), toml).expect("ce.toml");
    std::fs::write(dir.join("main.py"), WIDE).expect("main.py");
    common::build_index(&dir);
    dir
}

/// The best seam's benefit is linear in P_max, so declaring four
/// times the default must quadruple it — within the floor, since the
/// core prices in exact rationals and floors ONCE at the end, so
/// `floor(4y)` lands in `[4*floor(y), 4*floor(y)+3]` rather than on
/// the multiple. Comparing two runs instead of pinning one number
/// keeps the leg honest if the zone curve is ever retuned.
fn best_benefit(dir: &Path) -> i64 {
    let core = common::core_bin();
    let report = judge::run(dir, None, &core, (false, None, true)).expect("structure");
    let split = report.split.expect("split armed");
    split
        .candidates
        .iter()
        .map(|c| c.3)
        .chain(split.exempt.iter().map(|e| e.1))
        .max()
        .expect("the fixture must price at least one seam")
}

#[test]
fn a_declared_size_penalty_max_reaches_the_split_advisory() {
    let default = best_benefit(&fixture("structure-knob-default", None));
    let declared = best_benefit(&fixture("structure-knob-declared", Some(40)));
    assert!(default > 0, "the fixture must price a seam at the default");
    assert!(
        (default * 4..=default * 4 + 3).contains(&declared),
        "P_max 40 vs the core default 10: benefit is linear in P_max, so          {declared} must sit in {}..={}; outside that band the advisory          priced with a knob it never received",
        default * 4,
        default * 4 + 3
    );
}
