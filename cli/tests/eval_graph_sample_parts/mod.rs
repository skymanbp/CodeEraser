//! Hash-ranked stratified selection for the graph audit sample
//! (design §5): the pool-independent machinery, shared by the
//! generator and the CI gates so neither can drift from the other
//! (the G1 discipline). No RNG, no clock — ranks are sha256 over
//! domain-separated payloads (the eval_extract/freeze.rs precedent,
//! reworked in pure integer arithmetic), apportionment is integer
//! largest-remainder.

pub mod binding;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Pre-registered sampling constants (design §5, decisions D9/D10):
/// a floor of MIN_PER_LANG per language before EXTRA proportional
/// seats — pure proportionality would hand TS/Go ~2 seats each
/// (D2-4) — plus a per-language backup tail for denominator top-up
/// ("audit walks the frozen order into the backups until 100
/// answered rows").
pub const MIN_PER_LANG: u64 = 15;
pub const EXTRA: u64 = 25; // 5×15 + 25 = the plan-literal 100
pub const BACKUP_PER_LANG: u64 = 20; // 5×20 = the 100-row backup

/// Domain-separation tags. The rung domain is pre-registered now but
/// materializes only at 2f, when measured rungs exist (per-rung 15
/// overlap rows for minRung calibration — explicitly not a gate).
pub const SITE_DOMAIN: &str = "ce-graph-site-v1";
pub const AUDIT_DOMAIN: &str = "ce-graph-audit-v1";
pub const RUNG_DOMAIN: &str = "ce-graph-rung-v1";

/// The constants block frozen into the sample doc — one binding for
/// the generator and the gate.
pub fn constants() -> Value {
    json!({
        "min_per_lang": MIN_PER_LANG, "extra": EXTRA,
        "backup_per_lang": BACKUP_PER_LANG,
        "domains": {"site": SITE_DOMAIN, "audit": AUDIT_DOMAIN, "rung": RUNG_DOMAIN},
    })
}

/// One candidate with its full frozen identity. `nth` (the site's
/// per-line ordinal) entered site identity at 2b-iii; the design-§5
/// payload predates that and is extended by it — without nth, two
/// same-line sites of one kind and spec would collide into a single
/// rank id and verify() would refuse the whole sample.
#[derive(Clone)]
pub struct Site {
    pub corpus: String, // "self" for the enclosing repository
    pub commit: String,
    pub path: String,
    pub line: u64,
    pub nth: u64,
    pub kind: String,
    pub spec: String,
}

/// The '|'-joined hash payload. Every field before `spec` is
/// '|'-free by construction (corpus names, OIDs, tree paths,
/// integers, grammar kinds) and `spec` comes last, so the encoding
/// is injective. `lang` is derivable from the path and stays out —
/// the design-§5 field list plus nth, nothing more.
pub fn payload(s: &Site) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        s.corpus, s.commit, s.path, s.line, s.nth, s.kind, s.spec
    )
}

/// Domain-separated rank: sha256("<domain>|<payload>"), lowercase hex.
pub fn rank_of(domain: &str, s: &Site) -> String {
    let mut h = Sha256::new();
    h.update(domain.as_bytes());
    h.update(b"|");
    h.update(payload(s).as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Integer largest-remainder apportionment of `quota` seats over
/// strata proportional to their counts: base = ⌊q·n/T⌋, leftover
/// seats by descending q·n mod T (the same order as the fractional
/// parts — the denominators are equal), ties broken by ascending
/// key. Floats never enter; a stratum never exceeds its own count
/// while quota ≤ total.
pub fn largest_remainder(counts: &BTreeMap<String, u64>, quota: u64) -> BTreeMap<String, u64> {
    let total: u64 = counts.values().sum();
    assert!(quota <= total, "quota {quota} exceeds pool {total}");
    let mut out = BTreeMap::new();
    let mut rems: Vec<(u64, &String)> = Vec::new();
    let mut used = 0;
    for (key, n) in counts {
        out.insert(key.clone(), quota * n / total);
        rems.push((quota * n % total, key));
        used += quota * n / total;
    }
    rems.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    for (_, key) in rems.into_iter().take((quota - used) as usize) {
        *out.get_mut(key).expect("stratum") += 1;
    }
    out
}

pub fn lang_of_cell(cell: &str) -> &str {
    cell.split('/').next().expect("lang/kind")
}

/// Cell quotas from (lang/kind → pool count): the per-language floor
/// plus EXTRA largest-remainder seats over language totals, each
/// language quota then spread over its kinds by largest remainder —
/// the (lang,kind) stratification of design §5. Consumed by draw()
/// on the live pool AND recomputed by the CI gate from the frozen
/// slice summaries, so the allocation cannot drift from the universe
/// it was drawn on.
pub fn quotas_from_counts(cells: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let mut langs: BTreeMap<String, u64> = BTreeMap::new();
    for (cell, n) in cells {
        *langs.entry(lang_of_cell(cell).into()).or_insert(0) += n;
    }
    let extra = largest_remainder(&langs, EXTRA);
    let mut out = BTreeMap::new();
    for (lang, pool) in &langs {
        let quota = MIN_PER_LANG + extra[lang];
        assert!(*pool >= quota, "{lang}: pool {pool} below quota {quota}");
        let mine: BTreeMap<String, u64> = cells
            .iter()
            .filter(|(c, _)| lang_of_cell(c) == lang)
            .map(|(c, n)| (c.clone(), *n))
            .collect();
        out.append(&mut largest_remainder(&mine, quota));
    }
    out
}

/// Rebuild a Site from a frozen doc row; a missing or mistyped field
/// refuses instead of defaulting.
pub fn site_of(row: &Value) -> Result<Site, String> {
    let s = |k: &str| {
        row[k]
            .as_str()
            .map(str::to_string)
            .ok_or(format!("bad {k}"))
    };
    let n = |k: &str| row[k].as_u64().ok_or(format!("bad {k}"));
    // lang is validated like every frozen field but not stored: the
    // payload doc above keeps it out of identity, and its value is
    // cross-checked against the file table by binding.rs instead
    s("lang")?;
    Ok(Site {
        corpus: s("corpus")?,
        commit: s("commit")?,
        path: s("path")?,
        line: n("line")?,
        nth: n("nth")?,
        kind: s("kind")?,
        spec: s("spec")?,
    })
}

/// verify() of design §5: recompute both domain hashes from the
/// row's payload fields, refuse a mismatch, refuse a duplicate rank
/// id. Returns Err instead of panicking so the counterfactual gate
/// can prove refusal actually fires.
pub fn verify_row(row: &Value, seen: &mut BTreeSet<String>) -> Result<(), String> {
    let site = site_of(row)?;
    for (key, domain) in [("rank", SITE_DOMAIN), ("audit", AUDIT_DOMAIN)] {
        if row[key].as_str() != Some(rank_of(domain, &site).as_str()) {
            return Err(format!("{}:{}: {key} hash mismatch", site.path, site.line));
        }
    }
    match seen.insert(rank_of(SITE_DOMAIN, &site)) {
        true => Ok(()),
        false => Err(format!("{}:{}: duplicate id", site.path, site.line)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(pairs: &[(&str, u64)]) -> BTreeMap<String, u64> {
        pairs.iter().map(|(k, n)| (k.to_string(), *n)).collect()
    }

    #[test]
    fn largest_remainder_sums_and_breaks_ties_on_key() {
        let q = largest_remainder(&counts(&[("a", 1), ("b", 1), ("c", 98)]), 50);
        assert_eq!(q.values().sum::<u64>(), 50);
        // a and b tie on remainder; the lexicographically first bumps
        assert_eq!((q["a"], q["b"], q["c"]), (1, 0, 49));
    }

    #[test]
    fn quotas_floor_then_apportion() {
        // one kind per language keeps cell quota == language quota:
        // 3×15 floor + 25 extra over pools 20/800/180 (go gets the
        // tie-broken leftover seat, md 20, py 4)
        let cells = counts(&[("go/import_spec", 20), ("md/link", 800), ("py/import", 180)]);
        let q = quotas_from_counts(&cells);
        assert_eq!(q.values().sum::<u64>(), 3 * MIN_PER_LANG + EXTRA);
        assert_eq!(
            (q["go/import_spec"], q["md/link"], q["py/import"]),
            (16, 35, 19)
        );
    }

    #[test]
    fn domains_separate_and_nth_disambiguates() {
        let s = Site {
            corpus: "self".into(),
            commit: "deadbeef".into(),
            path: "a.md".into(),
            line: 3,
            nth: 0,
            kind: "link".into(),
            spec: "x|y".into(),
        };
        assert_ne!(rank_of(SITE_DOMAIN, &s), rank_of(AUDIT_DOMAIN, &s));
        let twin = Site {
            nth: 1,
            ..s.clone()
        };
        assert_ne!(payload(&s), payload(&twin));
        assert_ne!(rank_of(SITE_DOMAIN, &s), rank_of(SITE_DOMAIN, &twin));
    }
}
