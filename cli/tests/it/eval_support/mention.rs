//! K23 instrument (sealed criterion §7 K23, §2 ②, L5-F3/F17, L4-F10):
//! the tokenizer's two arms measured on one corpus with the product's
//! OWN emitters, the product's OWN domain and the product's OWN veto
//! channels — the L7-F6 rule: every number of a ledger row comes from
//! one implementation in one run. Two costs the `$` arm has and one it
//! must not have, the JS arm's collateral, the `$` run shapes, and the
//! two TEST-rule pins.

use codeeraser::mention::candidates::Decl;
use codeeraser::mention::conv::Conv;
use codeeraser::mention::conv::name::PathWords;
use codeeraser::mention::selfref::SelfText;
use codeeraser::mention::token::{FOLD_MIN_CHARS, emit, fold, runs, segments, whole_run_only};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The late-NUL PDFs the criterion names (§6): the second universe
/// leaves these suffixes out so the `.ai` cost is a column of its own.
const LATE_NUL_SUFFIXES: [&str; 2] = [".ai", ".eps"];

#[derive(Debug, Default, serde::Serialize)]
pub struct Ledger {
    /// ② the `$` arm's row cost: per file, distinct tokens emitter
    /// (iii) alone contributes, summed — over the full U and over U
    /// minus the late-NUL suffixes.
    pub union_rows_full: usize,
    pub union_rows_no_late_nul: usize,
    /// ① the `$` arm's advisory effect: domain declarations the veto
    /// takes under the real arms and would NOT take with emitter (iii)
    /// silent — all three channels asked on both sides (identity, the
    /// Rust fold, the file's own exception regions), as the producer
    /// asks them. Pre-registered 0 (L4-F3); every such row is named
    /// beside the count so a non-zero can be read, not just seen.
    pub union_advisory_diff: usize,
    pub union_advisory_rows: Vec<String>,
    /// The JS arm's collateral (L5-F3): distinct `$`-free pieces it
    /// silences across the corpus, of which no file in U spells the
    /// piece under the real arms, of which domain names — the last
    /// pre-registered 0.
    pub js_suppressed: usize,
    pub js_no_other_source: usize,
    pub js_domain: usize,
    /// `$`-bearing runs in JS-family files by shape.
    pub dollar_bare: usize,
    pub dollar_leading: usize,
    pub dollar_trailing: usize,
    pub dollar_inner: usize,
    /// Files of U under a `test` (singular) path component — the
    /// component the TEST rule deliberately lists; pre-registered 0
    /// on the four corpora (L4-F10).
    pub test_singular_files: usize,
    /// `<pkg>/{benches,examples}` directories whose files earn the
    /// Test bit — ripgrep pins exactly four (L5 review).
    pub pkg_test_dirs: BTreeSet<String>,
    /// bit 4's name-table half, hit per `lang:name` (L3-F6 ii).
    pub protocol_hits: BTreeMap<String, usize>,
}

/// One file of U read and tokenized under both arms.
struct Tokens<'t> {
    real: BTreeSet<&'t str>,
    /// Emitter (iii)'s own contribution: `union − whole-run`.
    union_only: BTreeSet<&'t str>,
}

fn tokens<'t>(rel: &str, text: &'t str) -> (Tokens<'t>, bool) {
    let js = whole_run_only(rel);
    let mut strict = BTreeSet::new();
    emit(text, true, &mut |t| {
        strict.insert(t);
    });
    let mut union = BTreeSet::new();
    emit(text, false, &mut |t| {
        union.insert(t);
    });
    let union_only: BTreeSet<&str> = union.difference(&strict).copied().collect();
    let real = if js { strict } else { union };
    (Tokens { real, union_only }, js)
}

fn late_nul(rel: &str) -> bool {
    LATE_NUL_SUFFIXES.iter().any(|s| rel.ends_with(s))
}

/// The judged domain keyed `(file, name)`, as `rates::declarations` reads it.
pub type Domain = BTreeMap<(String, String), Decl>;

type Spellers<'t> = BTreeMap<&'t str, BTreeSet<&'t str>>;

/// Who spells what, under each arm: identity tokens and — for tokens
/// of at least `FOLD_MIN_CHARS` — fold keys, exactly the two hashes
/// the store keeps per file (mod.rs). `strict` is the real arm with
/// emitter (iii) silent.
#[derive(Default)]
struct Spelling<'t> {
    real: Spellers<'t>,
    strict: Spellers<'t>,
    fold_real: BTreeMap<String, BTreeSet<&'t str>>,
    fold_strict: BTreeMap<String, BTreeSet<&'t str>>,
}

impl<'t> Spelling<'t> {
    fn spell(&mut self, rel: &'t str, tok: &'t str, strict: bool) {
        self.real.entry(tok).or_default().insert(rel);
        let folded = (tok.chars().count() >= FOLD_MIN_CHARS).then(|| fold(tok));
        if let Some(key) = &folded {
            self.fold_real.entry(key.clone()).or_default().insert(rel);
        }
        if strict {
            self.strict.entry(tok).or_default().insert(rel);
            if let Some(key) = folded {
                self.fold_strict.entry(key).or_default().insert(rel);
            }
        }
    }
}

/// The corpus read once: the spelling maps under both arms, the JS
/// arm's silenced pieces, and the per-file counters of the ledger.
fn spell<'t>(texts: &'t [(String, String)], l: &mut Ledger) -> (Spelling<'t>, Spellers<'t>) {
    let mut s = Spelling::default();
    let mut js_pieces: Spellers<'t> = BTreeMap::new();
    for (rel, text) in texts {
        let (t, js) = tokens(rel, text);
        if rel.split('/').rev().skip(1).any(|c| c == "test") {
            l.test_singular_files += 1;
        }
        for tok in &t.real {
            s.spell(rel, tok, !t.union_only.contains(tok));
        }
        if js {
            for tok in &t.union_only {
                js_pieces.entry(tok).or_default().insert(rel);
            }
            shapes(l, text);
        } else {
            l.union_rows_full += t.union_only.len();
            l.union_rows_no_late_nul += if late_nul(rel) { 0 } else { t.union_only.len() };
        }
    }
    (s, js_pieces)
}

/// The ledger of `root` over the universe `files` (the pinned formula's
/// members) and the domain `decls`.
pub fn ledger(root: &Path, files: &[String], decls: &Domain) -> Ledger {
    let texts: Vec<(String, String)> = files
        .iter()
        .filter_map(|rel| {
            let bytes = std::fs::read(root.join(rel)).ok()?;
            Some((rel.clone(), codeeraser::mention::decode(&bytes)?))
        })
        .collect();
    let mut l = Ledger::default();
    let (s, js_pieces) = spell(&texts, &mut l);
    let domain: BTreeSet<&str> = decls.keys().map(|(_, name)| name.as_str()).collect();
    l.js_suppressed = js_pieces.len();
    for tok in js_pieces.keys() {
        if !s.real.contains_key(tok) {
            l.js_no_other_source += 1;
            l.js_domain += usize::from(domain.contains(tok));
        }
    }
    let mut words = PathWords::new(root);
    let mut selfs: BTreeMap<&str, SelfText> = BTreeMap::new();
    for ((path, name), d) in decls {
        let real = vetoed(&s.real, &s.fold_real, &mut selfs, root, path, name, d);
        if real && !vetoed(&s.strict, &s.fold_strict, &mut selfs, root, path, name, d) {
            l.union_advisory_diff += 1;
            let spellers: Vec<&str> = s.real[name.as_str()]
                .iter()
                .copied()
                .filter(|p| *p != path)
                .collect();
            l.union_advisory_rows
                .push(format!("{}:{} ({}) via {:?}", path, d.line, name, spellers));
        }
        if d.conv & Conv::Protocol.bit() != 0 {
            *l.protocol_hits
                .entry(format!("{}:{}", d.lang.name(), name))
                .or_default() += 1;
        }
        if let Some(dir) = pkg_test_dir(&mut words, path) {
            l.pkg_test_dirs.insert(dir);
        }
    }
    l
}

/// The producer's veto (candidates.rs) over one arm's spelling: another
/// file spells the name; a Rust name of ≥2 segments and ≥
/// `FOLD_MIN_CHARS` has its fold key spelled by another file; the
/// declaring file's own exception regions spell it. Any yes vetoes.
fn vetoed<'t>(
    spellers: &Spellers<'t>,
    folds: &BTreeMap<String, BTreeSet<&'t str>>,
    selfs: &mut BTreeMap<&'t str, SelfText>,
    root: &Path,
    path: &'t str,
    name: &str,
    d: &Decl,
) -> bool {
    let others = |s: Option<&BTreeSet<&str>>| s.is_some_and(|s| s.iter().any(|p| *p != path));
    others(spellers.get(name))
        || (d.lang.name() == "rust"
            && segments(name) >= 2
            && name.chars().count() >= FOLD_MIN_CHARS
            && others(folds.get(&fold(name))))
        || selfs
            .entry(path)
            .or_insert_with(|| SelfText::read(root, path))
            .mentions(name)
}

/// `$` runs by shape: all-`$`, `$`-led, `$`-ended, `$` inside.
fn shapes(l: &mut Ledger, text: &str) {
    for run in runs(text).filter(|r| r.contains('$')) {
        if run.bytes().all(|b| b == b'$') {
            l.dollar_bare += 1;
        } else if run.starts_with('$') {
            l.dollar_leading += 1;
        } else if run.ends_with('$') {
            l.dollar_trailing += 1;
        } else {
            l.dollar_inner += 1;
        }
    }
}

/// The `<pkg>/{benches,examples}` prefix whose component alone gives a
/// file its Test bit — asked of the product's own path rule by a
/// monotone probe, never restated: the bit absent on the prefix before
/// the component and present with it. A TEST_DIRS component earlier
/// in the path already carries the bit (no transition is seen), a
/// package root is whatever the rule says it is, and no basename arm
/// fires on `probe.rs`.
fn pkg_test_dir(words: &mut PathWords, rel: &str) -> Option<String> {
    let comps: Vec<&str> = rel.split('/').collect();
    let dirs = &comps[..comps.len() - 1];
    let mut test = |prefix: &[&str]| {
        let probe = prefix
            .iter()
            .copied()
            .chain(["probe.rs"])
            .collect::<Vec<_>>()
            .join("/");
        words.bits(&probe) & Conv::Test.bit() != 0
    };
    (0..dirs.len())
        .filter(|&i| matches!(dirs[i], "benches" | "examples"))
        .find(|&i| !test(&dirs[..i]) && test(&dirs[..=i]))
        .map(|i| dirs[..=i].join("/"))
}
