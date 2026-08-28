//! The `ce graph --mentions` face (K23 census, L round piece (8)), run
//! as a reader would — the one road that prints the per-language
//! census, which no other leg exercised (L round review): the JSON
//! field names pinned (serde renames nothing, so a renamed counter
//! would change the `ce.mentions-report/0.2.0` payload while the
//! empty-`rates` unit test stayed green), the fold channel witnessed
//! on a fixture (the census unit test counts `fold: 0`; the self
//! corpus measures 23), and the console line's nine holes filled in
//! both languages — `i18n::line` degrades silently on a hole/argument
//! mismatch, so the substituted numbers are asserted, not merely the
//! absence of a literal `{}`.

use crate::common;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `HttpServer` is spelled by no other file, but `main.rs` spells
/// `HTTP_SERVER`, whose fold key `httpserver` is the declaration's —
/// the fold's second chance; `start_server` nothing spells. One tree
/// per leg: the legs run in parallel and `tmp` clears its directory.
fn tree(tag: &str) -> PathBuf {
    let root = common::tmp(&format!("mentions-face-{tag}"));
    common::write_doc(
        &root,
        "--- src/server.rs\npub struct HttpServer;\npub fn start_server() {}\n\
         --- src/main.rs\nmod server;\nfn main() {\n    let _ = HTTP_SERVER;\n}\n",
    );
    root
}

fn run(root: &Path, extra: &[&str], zh: bool) -> String {
    let db = root.join("scratch-index.db");
    let mut c = Command::new(env!("CARGO_BIN_EXE_ce"));
    c.arg("graph")
        .arg("--mentions")
        .arg("--db")
        .arg(&db)
        .args(extra)
        .arg(root)
        .env_remove("CE_LANG");
    if zh {
        c.env("CE_LANG", "zh");
    }
    let out = c.output().expect("run ce");
    assert!(out.status.success(), "{out:?}");
    String::from_utf8(out.stdout).expect("utf-8")
}

fn keys(v: &Value) -> Vec<&str> {
    v.as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn the_json_face_names_every_counter_and_witnesses_the_fold() {
    let root = tree("json");
    let doc: Value = serde_json::from_str(&run(&root, &["--format", "json"], false)).expect("json");
    assert_eq!(doc["schema"], "ce.mentions-report/0.2.0");
    let rust = &doc["rates"]["rust"];
    assert_eq!(keys(rust), ["declared", "unmentioned", "vetoed"]);
    assert_eq!(keys(&rust["declared"]), ["all", "exported"]);
    assert_eq!(keys(&rust["unmentioned"]), ["all", "exported"]);
    assert_eq!(
        keys(&rust["vetoed"]),
        ["collision_saved", "fold", "other", "self_text"]
    );
    // the domain: `HttpServer` (fold-vetoed), `start_server`, `main`,
    // the `server` mount — three unmentioned, one of them exported
    assert_eq!(
        *rust,
        serde_json::json!({
            "declared": { "all": 4, "exported": 2 },
            "unmentioned": { "all": 3, "exported": 1 },
            "vetoed": { "collision_saved": 0, "fold": 1, "other": 0, "self_text": 0 },
        })
    );
}

#[test]
fn the_console_line_fills_its_nine_holes_in_both_languages() {
    let root = tree("console");
    let doc: Value = serde_json::from_str(&run(&root, &["--format", "json"], false)).expect("json");
    let r = &doc["rates"]["rust"];
    let n = |v: &Value| v.as_u64().expect("count");
    let en = run(&root, &[], false);
    let want = format!(
        "  rust: {} declared ({} exported) — {} unmentioned ({} exported); vetoed by another file {} (of which {} only by a same-name declaration), by fold {}, by the file's own exceptions {}",
        n(&r["declared"]["all"]),
        n(&r["declared"]["exported"]),
        n(&r["unmentioned"]["all"]),
        n(&r["unmentioned"]["exported"]),
        n(&r["vetoed"]["other"]),
        n(&r["vetoed"]["collision_saved"]),
        n(&r["vetoed"]["fold"]),
        n(&r["vetoed"]["self_text"]),
    );
    assert!(
        en.lines().any(|l| l == want),
        "EN census line missing:\n{en}"
    );
    let zh = run(&root, &[], true);
    let line = zh
        .lines()
        .find(|l| l.starts_with("  rust："))
        .unwrap_or_else(|| panic!("ZH census line missing:\n{zh}"));
    assert!(
        line.contains("折叠否决 1") && !line.contains("{}"),
        "{line}"
    );
}
