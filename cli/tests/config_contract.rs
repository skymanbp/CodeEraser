//! ce.toml is DECLARATIVE, so a declaration that silently does
//! nothing is exactly the failure this repo exists to fight. Two
//! such holes pinned here: a Windows-spelled exclude glob that
//! compiled to a pattern matching nothing, and a threshold ladder
//! one reader refused while the other judged with an unreachable
//! warn arm.

use codeeraser::config::Config;
use codeeraser::scan::walk;

mod common;

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
