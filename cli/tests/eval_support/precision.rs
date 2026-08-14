//! Precision-contract machinery shared by the graph (M5-2h) and T3
//! (M5-3f) instruments: the cross-corpus aggregate, the per-corpus
//! and overall gates, and the frozen attention ledger — the four
//! shapes the ratchet caught the second family re-growing verbatim.
//! Family semantics (what counts as the gate denominator, how rows
//! map to languages, the threshold itself) stay with the family.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Cross-corpus aggregation of one precision family.
#[derive(Default)]
pub struct PrecisionAgg {
    pub correct: u64,
    pub wrong: u64,
    pub rows: u64,
    pub by_lang: BTreeMap<String, u64>,
}

impl PrecisionAgg {
    /// Absorb one doc's answered counters.
    pub fn absorb(&mut self, c: u64, w: u64) {
        self.correct += c;
        self.wrong += w;
    }
}

/// The correct/wrong counters of one doc's summary.
pub fn correct_wrong(doc: &Value) -> (u64, u64) {
    let n = |k: &str| doc["summary"]["verdicts"][k].as_u64().unwrap_or(0);
    (n("correct"), n("wrong"))
}

/// The per-corpus precision gate where the denominator reaches 5
/// (the EVAL-SET small-denominator stance: below it the corpus
/// publishes its numbers but sets no gate). `answered` = (correct,
/// wrong) — the pair travels together everywhere.
pub fn assert_corpus_precision(key: &str, answered: (u64, u64), denom: u64, gate: f64, tag: &str) {
    let (c, w) = answered;
    if denom < 5 {
        return;
    }
    let p = c as f64 / (c + w) as f64;
    assert!(
        p >= gate,
        "{key}: per-corpus precision {p:.3} < {gate} on denominator {denom} ({tag})"
    );
}

/// The family contract: row total, per-language floors, overall gate.
pub fn assert_overall_precision(
    agg: &PrecisionAgg,
    total: u64,
    lang_floor: u64,
    gate: f64,
    tag: &str,
) {
    assert_eq!(agg.rows, total, "judged rows across corpora ({tag})");
    for (lang, n) in &agg.by_lang {
        assert!(
            *n >= lang_floor,
            "{lang}: {n} judged rows < per-lang floor {lang_floor}"
        );
    }
    let overall = agg.correct as f64 / (agg.correct + agg.wrong) as f64;
    assert!(
        overall >= gate,
        "overall precision {overall:.3} < {gate} ({tag})"
    );
}

/// G8/T-G8: an existing doc's attention ledger is frozen — growth
/// needs the family's explicit blessing variable.
pub fn assert_ledger_frozen(
    path: &str,
    current: &[Value],
    attention: &dyn Fn(&[Value]) -> BTreeSet<String>,
    accept_env: &str,
) {
    let Ok(old) = std::fs::read_to_string(path) else {
        return;
    };
    let old: Value = serde_json::from_str(&old).expect("old doc");
    let frozen = attention(old["rows"].as_array().expect("rows"));
    let grown: Vec<String> = attention(current).difference(&frozen).cloned().collect();
    assert!(
        grown.is_empty() || std::env::var(accept_env).as_deref() == Ok("1"),
        "attention ledger grew ({grown:?}) — regressions need {accept_env}=1"
    );
}

/// One count into a string-keyed tally — the map-accumulation throat
/// the summarizers share (an amount of 1 is the common case).
pub fn tally_add(map: &mut BTreeMap<String, u64>, key: &str, n: u64) {
    *map.entry(key.to_string()).or_insert(0) += n;
}

/// One corpus doc opened for its family gate: loaded, keyed by its
/// doc-name suffix (self when bare).
pub fn load_corpus_doc(path: &str, family: &str) -> (String, Value) {
    let doc = super::load(path);
    let key = super::doc_suffix(path, family).unwrap_or_else(|| "self".into());
    (key, doc)
}

/// The audited truth of one rank — sampled but unaudited is a hole,
/// never a default.
pub fn truth_of<'a>(corpus: &str, rank: &str, truths: &BTreeMap<&str, &'a str>) -> &'a str {
    truths
        .get(rank)
        .copied()
        .unwrap_or_else(|| panic!("{corpus}/{rank}: not in the audit"))
}

/// truth_of plus the stored row's echo of it — the lookup and the
/// drift assert both families run back to back.
pub fn assert_truth_echo<'a>(
    corpus: &str,
    rank: &str,
    row: &Value,
    truths: &BTreeMap<&str, &'a str>,
) -> &'a str {
    let truth = truth_of(corpus, rank, truths);
    assert_eq!(
        row["truth"].as_str(),
        Some(truth),
        "{corpus}/{rank}: truth echo drifted"
    );
    truth
}

/// The row-verdict re-derivation both families end every row check
/// with: truth echoed against the audit, verdict recomputed from the
/// row's own facts through the family's recompute closure.
pub fn assert_row_verdict(
    corpus: &str,
    rank: &str,
    row: &Value,
    truths: &BTreeMap<&str, &str>,
    recompute: impl FnOnce(&str) -> &'static str,
) {
    let truth = assert_truth_echo(corpus, rank, row, truths);
    assert_verdict_recomputed(corpus, rank, row, recompute(truth));
}

/// One scored doc opened for its family gate: loaded, keyed,
/// structurally checked, answered counters absorbed.
pub fn open_scored_doc(
    path: &str,
    family: &str,
    sample: &Value,
    agg: &mut PrecisionAgg,
    check: impl FnOnce(&str, &Value, &Value),
) -> (String, Value, u64, u64) {
    let (key, doc) = load_corpus_doc(path, family);
    check(&key, &doc, sample);
    let (c, w) = correct_wrong(&doc);
    agg.absorb(c, w);
    (key, doc, c, w)
}

/// The tamper batteries' shared opening: the family's sample doc and
/// its zod pristine doc (the largest corpus exercises every shape).
pub fn zod_pristine(family: &str, sample_name: &str) -> (Value, Value) {
    (
        super::load(&super::eval_doc(sample_name)),
        super::load(&super::eval_doc(&format!("{family}-zod"))),
    )
}

/// Tally one string field of every row into a map.
pub fn tally_field(rows: &[Value], field: &str, map: &mut BTreeMap<String, u64>) {
    for r in rows {
        tally_add(map, r[field].as_str().expect(field), 1);
    }
}

/// The G6/T-G6 discipline: the stored verdict must equal the one its
/// own row re-derives.
pub fn assert_verdict_recomputed(corpus: &str, rank: &str, row: &Value, recomputed: &str) {
    assert_eq!(
        row["verdict"].as_str(),
        Some(recomputed),
        "{corpus}/{rank}: stored verdict contradicts its own row (G6)"
    );
}

/// The G1 discipline: the stored summary must equal the one the
/// family's own scorer re-derives from the rows.
pub fn assert_summary_rederived(
    corpus: &str,
    doc: &Value,
    rescore: impl FnOnce(&[Value]) -> Value,
) {
    let rows = doc["rows"].as_array().expect("rows");
    assert_eq!(
        doc["summary"],
        rescore(rows),
        "{corpus}: summary drifted from rows (G1)"
    );
}

/// The zod tamper-battery frame: sample + pristine loaded, the
/// table-driven half run, then the family's bespoke cases against
/// the same bound checker.
pub fn assert_zod_battery(
    family: &str,
    sample_name: &str,
    check_with: &dyn Fn(&Value, &Value),
    mutations: &[(&str, &str, &str)],
    bespoke: impl FnOnce(&Value, &dyn Fn(&Value) -> bool),
) {
    let (sample_doc, pristine) = zod_pristine(family, sample_name);
    let check = |doc: &Value| check_with(doc, &sample_doc);
    super::assert_tampering_refused(&pristine, mutations, &check);
    let refused = |doc: &Value| super::doc_refused(doc, &check);
    bespoke(&pristine, &refused);
}

/// Verdict-class conservation of one doc: the summary tallies sum to
/// the row count and no class falls outside the family vocabulary.
pub fn assert_verdict_conservation(corpus: &str, doc: &Value, classes: &[&str]) {
    let rows = doc["rows"].as_array().expect("rows");
    let verdicts = doc["summary"]["verdicts"].as_object().expect("verdicts");
    let total: u64 = verdicts.values().map(|v| v.as_u64().expect("n")).sum();
    assert_eq!(total, rows.len() as u64, "{corpus}: conservation (G3)");
    for key in verdicts.keys() {
        assert!(
            classes.contains(&key.as_str()),
            "{corpus}: unknown verdict class {key}"
        );
    }
}

/// The family contract walk: every frozen doc opened through the
/// family's own corpus gate, then the overall gate.
pub fn assert_family_contract(
    family: &str,
    sample_name: &str,
    gate: f64,
    tag: &str,
    mut corpus_gate: impl FnMut(&str, &Value, &mut PrecisionAgg),
) {
    let docs = super::assert_frozen_corpus_set(family);
    let sample = super::load(&super::eval_doc(sample_name));
    let mut agg = PrecisionAgg::default();
    for path in &docs {
        corpus_gate(path, &sample, &mut agg);
    }
    assert_overall_precision(&agg, 100, 15, gate, tag);
}

/// One precision family's constants, bundled (the E01 params cap and
/// the twin-test scaffold both dissolve into this one spec).
pub struct FamilySpec<'a> {
    pub family: &'a str,
    pub sample: &'a str,
    pub gate: f64,
    pub tag: &'a str,
    pub mutations: &'a [(&'a str, &'a str, &'a str)],
}

/// The whole family battery: contract walk plus the zod tamper
/// battery — each precision family is exactly one call to this,
/// passing its (corpus, doc, sample) checker; the zod binding lives
/// here, once.
pub fn assert_precision_family(
    spec: &FamilySpec,
    corpus_gate: impl FnMut(&str, &Value, &mut PrecisionAgg),
    check_doc: &dyn Fn(&str, &Value, &Value),
    bespoke: impl FnOnce(&Value, &dyn Fn(&Value) -> bool),
) {
    assert_family_contract(spec.family, spec.sample, spec.gate, spec.tag, corpus_gate);
    assert_zod_battery(
        spec.family,
        spec.sample,
        &|doc, s| check_doc("zod", doc, s),
        spec.mutations,
        bespoke,
    );
}
