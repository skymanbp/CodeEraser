//! K23 (sealed criterion §7): the four external corpora and the self
//! corpus under the mention instrument — the universe pinned to the
//! walk's formula, the per-language census, the tokenizer's two arms
//! costed with the product's own emitters against the product's own
//! domain, and the pre-registered zeros asserted (a non-zero is the
//! one shape that would mean an implementation defect, S-A22 law):
//!   ① the `$` union arm changes no advisory (L4-F3);
//!   ② the JS arm silences no domain name that nothing else spells
//!      (L5-F3);
//!   ③ no EXTERNAL corpus has a `test` (singular) path component
//!      (L4-F10 — the self corpus has them and only reports the count);
//!   ④ ripgrep's package-root Test rule hits exactly its four
//!      `benches`/`examples` directories (L5 review).
//! The external legs are `--ignored`: the corpora live under
//! `.ce-eval/corpora/<name>` (tips in docs/EVAL-SET-M5-3.md, checked
//! here) and never in CI. Every index is a scratch database — nothing
//! is written into a corpus. Run:
//!   cargo test --release --test it -- eval_mention --ignored --nocapture

use crate::common;
use crate::eval_support::mention::{Ledger, ledger};
use crate::mention_universe::{Formula, formula};
use codeeraser::mention::rates::{census, declarations};
use serde_json::json;
use std::path::Path;

/// The pinned tips (docs/EVAL-SET-M5-3.md, 语料树钉定): a ledger row
/// on any other tree would be a different corpus.
const CORPORA: [(&str, &str); 4] = [
    ("cobra", "adbc8813901bba65827259daa8e22ff94ec1f30e"),
    ("requests", "8068356288978c4f54661ae6f95afe0e0831885e"),
    ("ripgrep", "3fce3b5bb0236da2df6d99672afb8a719642eca7"),
    ("zod", "912f0f51b0ced654d0069741e7160834dca742ee"),
];

const RIPGREP_TEST_DIRS: [&str; 4] = [
    "crates/globset/benches",
    "crates/grep/examples",
    "crates/ignore/examples",
    "crates/searcher/examples",
];

/// One corpus through the instrument: scratch index, mention pass,
/// formula pin, census, ledger; the ledger printed as one JSON line
/// (the booklet's numbers are copied from this output, never typed —
/// every term of the formula rides, so `listed − Σ terms = U` closes
/// inside the line).
fn measure(name: &str, root: &Path) -> (Formula, Ledger) {
    let scratch = common::tmp(&format!("eval-mention-{name}"));
    let (idx, _db) = codeeraser::dedup::refreshed_index(root, Some(scratch.join("index.db")))
        .expect("scratch index");
    let stats = codeeraser::mention::refresh(root, &idx).expect("mention pass");
    let f = formula(root);
    f.assert_matches(&stats);
    let rates = census(root, &idx).expect("census");
    let decls = declarations(&idx).expect("domain");
    let files = f.files.clone();
    let l = ledger(root, &files, &decls);
    let mut universe = serde_json::to_value(&f).expect("formula terms");
    universe["U"] = json!(f.universe());
    println!(
        "{}",
        json!({
            "corpus": name,
            "universe": universe,
            "mention": stats,
            "rates": rates,
            "ledger": l,
        })
    );
    // ① holds as pre-registered (L4-F3) on all five trees once the
    // instrument asks all three veto channels on both arms. An
    // identity-only reading of the same run had shown two zod rows
    // (`ZodBase64URL`, `ZodExactOptional` — bare names spelled by no
    // other file, `$`-twins spelled in the docs); both are vetoed
    // with the arm silent too, by their own file's string literals
    // (`$constructor("ZodBase64URL", …)`), so the arm changes no
    // advisory anywhere. The rows are printed, not just counted, so a
    // non-zero can be read.
    assert!(
        l.union_advisory_rows.is_empty(),
        "{name}: ① the `$` arm changed the advisory: {:?}",
        l.union_advisory_rows
    );
    assert_eq!(
        l.js_domain, 0,
        "{name}: ② the JS arm silenced a domain name"
    );
    (f, l)
}

#[test]
fn the_self_corpus_holds_the_preregistered_zeros() {
    let root = common::repo_root();
    let (f, l) = measure("self", &root);
    assert!(f.universe() > 500, "the repository itself is the corpus");
    assert!(
        l.pkg_test_dirs.is_empty(),
        "no package-root benches/examples here: {:?}",
        l.pkg_test_dirs
    );
}

#[test]
#[ignore = "needs the pinned corpora under .ce-eval/corpora"]
fn the_external_corpora_hold_the_preregistered_zeros() {
    let base = common::repo_root().join(".ce-eval/corpora");
    for (name, tip) in CORPORA {
        let root = base.join(name);
        assert!(
            root.is_dir(),
            "{}: clone the corpus at {tip}",
            root.display()
        );
        let head = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git rev-parse");
        assert_eq!(
            String::from_utf8_lossy(&head.stdout).trim(),
            tip,
            "{name}: not the pinned tip"
        );
        let (_f, l) = measure(name, &root);
        assert_eq!(
            l.test_singular_files, 0,
            "{name}: ③ a `test` (singular) component"
        );
        if name == "ripgrep" {
            let got: Vec<&str> = l.pkg_test_dirs.iter().map(String::as_str).collect();
            assert_eq!(
                got, RIPGREP_TEST_DIRS,
                "④ the package-root Test rule's witnesses"
            );
        }
    }
}
