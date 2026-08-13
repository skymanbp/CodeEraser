//! Materialization + scoring engine for the precision instrument
//! (design §5). The generator and every CI gate score through the
//! SAME functions here (the G1 discipline), so a summary can never
//! drift from its rows without reddening.

use crate::eval_support::{
    TRUTH_KEYWORDS, classify_path, content_sha, git_run, lang_of, str_pairs,
};
use codeeraser::graph::ladder::{self, Outcome, Reason, Scope, Site};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The pinned tree, materialized: the frozen in-scope file set plus
/// the resolver configs (configs carry no slice identity of their
/// own — the tree OID pins them transitively).
pub struct Mat {
    pub root: PathBuf,
    pub files: BTreeSet<String>,
    pub configs: Vec<String>,
}

impl Mat {
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            files: &self.files,
            configs: &self.configs,
            root: &self.root,
        }
    }
}

/// Write the pinned tree's in-scope files + resolver configs under a
/// scratch root. Every in-scope file's text must hash to the frozen
/// slice identity — a polluted materialization never scores (the
/// G12 stance: a degraded run is refused, not recorded).
pub fn materialize(name: &str, tip: &str, slice: &Value) -> Mat {
    let frozen = str_pairs(slice, "files", "path", "sha256");
    let root = std::env::temp_dir().join(format!("ce-graph-prec-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let listing = git_run(
        &["ls-tree", "-r", "--full-tree", "--name-only", "-z", tip],
        false,
    );
    let mut m = Mat {
        root,
        files: BTreeSet::new(),
        configs: Vec::new(),
    };
    for path in listing.split('\0').filter(|p| !p.is_empty()) {
        let in_scope = classify_path(path).is_ok();
        let is_cfg = codeeraser::graph::store::is_resolver_config(Path::new(path));
        if !in_scope && !is_cfg {
            continue;
        }
        let text = git_run(&["show", &format!("{tip}:{path}")], false);
        if in_scope {
            let want = frozen
                .get(path)
                .unwrap_or_else(|| panic!("{path}: in scope but not in the frozen slice"));
            assert_eq!(
                &content_sha(&text),
                want,
                "{path}: materialized text drifted from the frozen slice"
            );
            m.files.insert(path.to_string());
        } else {
            m.configs.push(path.to_string());
        }
        let dst = m.root.join(path);
        std::fs::create_dir_all(dst.parent().expect("parent")).expect("mkdir");
        std::fs::write(&dst, text.as_bytes()).expect(path);
    }
    m
}

/// One judged sample row: the sampled identity, the resolver's
/// answer, and the verdict against the frozen truth.
pub fn judge_row(row: &Value, truth: &str, scope: &Scope) -> Value {
    let lang = row["lang"].as_str().expect("lang");
    let site = Site {
        kind: row["kind"].as_str().expect("kind"),
        from: row["path"].as_str().expect("from"),
        spec: row["spec"].as_str().expect("spec"),
        line: usize::try_from(row["line"].as_u64().expect("line")).expect("line"),
    };
    let out = ladder::resolve(lang_of(lang), &site, scope);
    let (answered, external, rung, reason) = shape(&out);
    json!({
        "rank": row["rank"], "path": row["path"], "line": row["line"],
        "nth": row["nth"], "kind": row["kind"], "spec": row["spec"],
        "lang": lang, "answered": answered, "external": external,
        "rung": rung, "reason": reason, "truth": truth,
        "verdict": verdict_of(truth, answered.as_deref(), external),
    })
}

/// Outcome → (normalized answer, external?, rung, refusal reason).
/// A package answer at the repo root normalizes to "." (the audited
/// cobra self-import row); a slugless section answer is the
/// file-level degrade and answers the file.
fn shape(out: &Outcome) -> (Option<String>, bool, Option<u8>, Option<&'static str>) {
    match out {
        Outcome::Resolved { path, rung } => (Some(path.clone()), false, Some(*rung), None),
        Outcome::ResolvedPackage { dir, rung } => {
            let norm = if dir.is_empty() {
                ".".into()
            } else {
                dir.clone()
            };
            (Some(norm), false, Some(*rung), None)
        }
        Outcome::ResolvedSection { path, slug, rung } => {
            let norm = match slug {
                Some(s) => format!("{path}#{s}"),
                None => path.clone(),
            };
            (Some(norm), false, Some(*rung), None)
        }
        Outcome::External { rung } => (None, true, Some(*rung), None),
        Outcome::Unresolved(r) => (None, false, None, Some(reason_str(*r))),
    }
}

fn reason_str(r: Reason) -> &'static str {
    match r {
        Reason::Dynamic => "dynamic",
        Reason::AmbiguousPaths => "ambiguous_paths",
        Reason::AmbiguousRoot => "ambiguous_root",
        Reason::AmbiguousWorkspace => "ambiguous_workspace",
        Reason::AmbiguousExports => "ambiguous_exports",
        Reason::Macro => "macro",
        Reason::ConfigDepth => "config_depth",
        Reason::OutOfScope => "out_of_scope",
        Reason::Unsupported => "unsupported",
    }
}

/// The five-way verdict (design D4 + G3 conservation vocabulary).
/// An in-corpus answer matches its truth exactly, or at file level
/// when the resolver made no unit claim (the ladder is file-level by
/// design at 2f; GT records definition units as extra precision —
/// the R5 symbol-binding rung is future work with its own stance).
/// External is an ANSWER: claiming external for an in-corpus truth
/// is wrong, not missed.
pub fn verdict_of(truth: &str, answered: Option<&str>, external: bool) -> &'static str {
    if let Some(a) = answered {
        let file_level = !a.contains('#') && truth.split('#').next() == Some(a);
        return if truth == a || file_level {
            "correct"
        } else {
            "wrong"
        };
    }
    if external {
        return if truth == "external" {
            "external_ok"
        } else {
            "wrong"
        };
    }
    if TRUTH_KEYWORDS.contains(&truth) {
        "unresolved_ok"
    } else {
        "missed"
    }
}

pub const VERDICTS: [&str; 5] = ["correct", "wrong", "missed", "external_ok", "unresolved_ok"];

/// Summary re-derived from the judged rows — generator and gate call
/// this one scorer (G1). The cut table publishes precision per rung
/// ceiling: at ceiling k, answers above k count as refusals.
pub fn rescore(rows: &[Value]) -> Value {
    let mut by_lang: BTreeMap<&str, BTreeMap<&str, u64>> = BTreeMap::new();
    let mut totals: BTreeMap<&str, u64> = BTreeMap::new();
    let mut in_truth = 0u64;
    for r in rows {
        let v = r["verdict"].as_str().expect("verdict");
        let lang = r["lang"].as_str().expect("lang");
        *totals.entry(v).or_insert(0) += 1;
        *by_lang.entry(lang).or_default().entry(v).or_insert(0) += 1;
        let truth = r["truth"].as_str().expect("truth");
        if !TRUTH_KEYWORDS.contains(&truth) {
            in_truth += 1;
        }
    }
    let n = |m: &BTreeMap<&str, u64>, k: &str| m.get(k).copied().unwrap_or(0);
    let cut: Vec<Value> = (1..=5u8)
        .map(|k| {
            let (mut c, mut w) = (0u64, 0u64);
            for r in rows {
                let within = r["rung"].as_u64().is_some_and(|g| g <= u64::from(k));
                match (
                    r["verdict"].as_str(),
                    within && !r["external"].as_bool().unwrap_or(false),
                ) {
                    (Some("correct"), true) => c += 1,
                    (Some("wrong"), true) if r["answered"].is_string() => w += 1,
                    _ => {}
                }
            }
            json!([k, c, w, ratio(c, w)])
        })
        .collect();
    let (c, w) = (n(&totals, "correct"), n(&totals, "wrong"));
    json!({
        "verdicts": totals,
        "precision": ratio(c, w),
        "recall": if in_truth == 0 { Value::Null } else { json!(c as f64 / in_truth as f64) },
        "in_corpus_truths": in_truth,
        "by_lang": by_lang,
        "cut": cut,
    })
}

fn ratio(c: u64, w: u64) -> Value {
    if c + w == 0 {
        Value::Null
    } else {
        json!(c as f64 / (c + w) as f64)
    }
}

/// The anti-cowardice ledger over the WHOLE frozen universe, not the
/// sample (design §5 三数): per (lang,kind,rung) resolution counts
/// and the refusal buckets. The resolution rate is a recall CEILING
/// — constructs the detector cannot see are not in any denominator.
pub fn universe_ledger(m: &Mat) -> Value {
    let scope = m.scope();
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    let mut refused: BTreeMap<&str, u64> = BTreeMap::new();
    let (mut total, mut answered, mut r1) = (0u64, 0u64, 0u64);
    for path in &m.files {
        let code = classify_path(path).expect("in scope");
        let text = std::fs::read_to_string(m.root.join(path)).expect(path);
        for site in codeeraser::graph::sites::detect(&text, lang_of(code)) {
            total += 1;
            let at = Site {
                kind: site.kind,
                from: path,
                spec: &site.spec,
                line: site.line,
            };
            let out = ladder::resolve(lang_of(code), &at, &scope);
            let (a, ext, rung, reason) = shape(&out);
            if a.is_some() || ext {
                answered += 1;
                let g = rung.expect("answered rung");
                if g == 1 {
                    r1 += 1;
                }
                *by.entry(format!("{code}/{}/r{g}", site.kind)).or_insert(0) += 1;
            } else {
                *refused.entry(reason.expect("reason")).or_insert(0) += 1;
            }
        }
    }
    json!({
        "total_sites": total,
        "resolution_rate": if total == 0 { Value::Null } else { json!(answered as f64 / total as f64) },
        "r0_share": if answered == 0 { Value::Null } else { json!(r1 as f64 / answered as f64) },
        "resolution_by": by,
        "unresolved_by": refused,
    })
}
