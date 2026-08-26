//! The rulepack's scan half end to end (plan v2.13 ① P3, 3.2.0):
//! one fixture, one class, the same 60-line function twice — inside
//! the class its fn line sits at 80 and the row grades clean; on the
//! global table it warns at 50, and the finding names the line it was
//! measured against. The classed run rides the whole judged road
//! (Classes → rowClasses/gradeOverrides → the core → findings_from),
//! and analyze_judged's pinned-mirror ensure proves the local
//! per-class evaluate equal to the wire on every such run.

use crate::common;
use codeeraser::scan;

#[test]
fn a_class_moves_its_files_fn_ladder_and_the_mirror_holds() {
    let body: String = (0..58).map(|i| format!("    let v{i} = {i};\n")).collect();
    let long_fn = format!("fn long() {{\n{body}}}\n");
    let fx = common::fixture(
        "scan-classes",
        &[
            // the class's own hard line rides too: a warn at 80 over
            // the inherited fail 75 is exactly the ladder the load
            // throat refuses (C8) — a classed ladder must climb
            (
                "ce.toml",
                "[[rules.class]]\nname = \"gen\"\nglobs = [\"gen/**\"]\n[rules.class.knobs]\nfn_lines_warn = 80\nfn_lines_fail = 90\n",
            ),
            ("gen/a.rs", &long_fn),
            ("src/b.rs", &long_fn),
        ],
    );
    let core = common::gates::core_bin();
    let (files, findings, _summary, fail) = scan::analyze_judged(&fx.dir, &core).expect("judged");
    assert_eq!(files.len(), 2, "both files measured");
    assert!(!fail, "no hard line breached");
    let fn_lines: Vec<(&str, usize, usize)> = findings
        .iter()
        .filter(|f| f.rule == "fn-lines")
        .map(|f| (f.file.as_str(), f.value, f.threshold))
        .collect();
    assert_eq!(
        fn_lines,
        vec![("src/b.rs", 60, 50)],
        "the classed file's 60-line fn is clean under its 80 line; the global one warns at 50"
    );
}
