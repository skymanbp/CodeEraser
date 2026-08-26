//! ce.toml is DECLARATIVE, so a declaration that silently does
//! nothing is exactly the failure this repo exists to fight. Two
//! such holes pinned here: a Windows-spelled exclude glob that
//! compiled to a pattern matching nothing, and a threshold ladder
//! one reader refused while the other judged with an unreachable
//! warn arm.

use crate::common;
use codeeraser::config::Config;
use codeeraser::scan::walk;

/// '\' is an escape in override/gitignore syntax and the candidates
/// are '/'-spelled, so `src\generated\*.rs` — the natural spelling on
/// this project's primary platform — used to exclude nothing at all,
/// with no error to say so.
#[test]
fn windows_spelled_exclude_glob_excludes() {
    let fx = common::fixture(
        "config-contract-exclude",
        &[
            ("src/generated/api.rs", "fn generated() {}\n"),
            ("src/keep.rs", "fn kept() {}\n"),
        ],
    );
    let kept = |globs: &[String]| -> Vec<String> {
        walk::collect(&fx.dir, globs)
            .expect("walk")
            .iter()
            .map(|p| walk::rel_str(&fx.dir, p))
            .collect()
    };
    assert_eq!(kept(&[]).len(), 2, "both files are in scope unexcluded");
    assert_eq!(kept(&["src\\generated\\*.rs".to_string()]), ["src/keep.rs"]);
    // the '/'-spelled form was never broken and stays exact
    assert_eq!(kept(&["src/generated/*.rs".to_string()]), ["src/keep.rs"]);
}

/// One config, two readers: scan/wire.rs::grade_rows refused
/// `fail < warn` while the report.rs mirror judged on happily, so
/// `ce scan` exited 2 on a ce.toml the MCP scan tool served a full
/// report from — a report whose warn arm could never fire.
#[test]
fn incoherent_threshold_ladder_is_refused_by_both_readers() {
    let fx = common::fixture(
        "config-contract-ladder",
        &[
            ("ce.toml", "[thresholds]\nfile_lines_warn = 800\n"),
            ("src/a.rs", "fn a() {}\n"),
        ],
    );
    let err = Config::load(&fx.dir).expect_err("fail 750 sits below warn 800");
    assert!(err.contains("file_lines_warn"), "{err}");
    // the shared measurement walk (score/structure reuse) dies on
    // the same config, not just the wire path — measure() is where
    // every scan surface loads ce.toml (batch-7 slice 8)
    let Err(err) = codeeraser::scan::measure(&fx.dir) else {
        panic!("the measurement walk must refuse the same config the wire path does");
    };
    assert!(err.to_string().contains("file_lines_warn"), "{err}");
    // fail 0 = "no hard line" stays legal at any warn line
    std::fs::write(
        fx.dir.join("ce.toml"),
        "[thresholds]\nfile_lines_warn = 800\nfile_lines_fail = 0\n",
    )
    .expect("ce.toml");
    Config::load(&fx.dir).expect("no hard line is coherent");
}

/// The rulepack's load throat (plan v2.13 ①): a class ladder is
/// judged on its EFFECTIVE lines through the same predicate the
/// global table answers to (C8), the fence refuses a 65th class, a
/// twice-declared name refuses, and serde names a missing key — every
/// refusal loud, never a class that silently matches nothing.
#[test]
fn rulepack_classes_refuse_at_the_load_throat() {
    let fx = common::fixture("config-contract-classes", &[("src/a.rs", "fn a() {}\n")]);
    let load = |toml: &str| {
        std::fs::write(fx.dir.join("ce.toml"), toml).expect("ce.toml");
        Config::load(&fx.dir)
    };
    let class = |name: &str, knobs: &str| {
        format!(
            "[[rules.class]]\nname = \"{name}\"\nglobs = [\"cli/tests/**\"]\n[rules.class.knobs]\n{knobs}\n"
        )
    };
    // C8: a class warn line above the INHERITED global hard line
    let err = load(&class("tests", "file_lines_warn = 800")).expect_err("800 > the global 750");
    assert!(
        err.contains("tests") && err.contains("file_lines_warn"),
        "{err}"
    );
    // the same class with its own hard line climbs, and loads
    let cfg = load(&class(
        "tests",
        "file_lines_warn = 800\nfile_lines_fail = 900",
    ))
    .expect("coherent");
    assert_eq!(
        cfg.rules.class[0]
            .effective(&cfg.thresholds)
            .file_lines_fail,
        900
    );
    let err = load(&(class("tests", "") + &class("tests", ""))).expect_err("declared twice");
    assert!(err.contains("declared twice"), "{err}");
    let err = load("[[rules.class]]\nglobs = [\"x/**\"]\n").expect_err("name is required");
    assert!(err.contains("name"), "{err}");
    let err = load("[[rules.class]]\nname = \"empty\"\nglobs = []\n").expect_err("no globs");
    assert!(err.contains("no globs"), "{err}");
    let many: String = (0..65).map(|i| class(&format!("c{i}"), "")).collect();
    let err = load(&many).expect_err("the fence");
    assert!(err.contains("fence"), "{err}");
    assert!(
        load(&many[..many.len() - class("c64", "").len()]).is_ok(),
        "64 classes load"
    );
}
