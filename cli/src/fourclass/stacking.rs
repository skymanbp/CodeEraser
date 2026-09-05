//! Stacking evidence for the M4 judgment rule: unit keys newly
//! DUPLICATED on the after side, shipped as `[key hash, start, end]`
//! rows — one per after-side occurrence — and nothing text-shaped
//! (ADR-002 A6). The spans arrived at 7.0.0 (O47): with hashes alone
//! the core could see THAT a key was duplicated, never WHERE, so the
//! rule fired on twenty novel lines anywhere in a file that gained a
//! duplicate key; with spans it asks whether the novel mass landed
//! inside a duplicated unit. Split from batch.rs when the
//! attack-review fixes pushed that file past the 300-line budget (E01:
//! split before exemption).

use super::batch::PairInput;
use super::units;
use crate::dedup::tokens::fnv1a;
use std::collections::HashMap;

/// Every after-side occurrence of a unit key whose after-side count
/// rises to >= 2 and above its before-side count, as ascending
/// `[fnv1a(key), start, end]` rows. Only TOP-LEVEL named units count:
/// a method nested in two different classes shares its flat key
/// legitimately, and an anonymous closure has no stacking identity at
/// all — both were measured false-positive sources on the real-edit
/// corpus (contracts/eval/fpr-fourclass-v1.json flagged 8/600 before
/// this scoping, 0/600 after). "impl " units are CONTAINERS (they
/// exist so methods are not top-level), never stacking identities
/// themselves: a type's inherent and trait impls, or split inherent
/// impls, coexist in perfectly normal Rust — the FPR replay flagged
/// exactly that shape when they were counted (attack review F7).
pub fn dup_spans(input: &PairInput) -> Vec<[u64; 3]> {
    let before = top_level_spans(input.before, input.lang);
    let mut out: Vec<[u64; 3]> = top_level_spans(input.after, input.lang)
        .into_iter()
        .filter(|(k, spans)| spans.len() >= 2 && spans.len() > before.get(k).map_or(0, Vec::len))
        .flat_map(|(k, spans)| {
            let hash = fnv1a(k.as_bytes());
            spans
                .into_iter()
                .map(move |(s, e)| [hash, s as u64, e as u64])
        })
        .collect();
    out.sort_unstable();
    out
}

/// Top-level named units by key, each with its 1-based inclusive
/// spans in source order.
fn top_level_spans(
    text: &str,
    lang: crate::scan::lang::Lang,
) -> HashMap<String, Vec<(usize, usize)>> {
    let all = units::segments(text, lang);
    let top_level = |u: &units::Unit| {
        !all.iter().any(|v| {
            (v.start_line < u.start_line && u.end_line <= v.end_line)
                || (v.start_line <= u.start_line && u.end_line < v.end_line)
        })
    };
    all.iter()
        .filter(|u| {
            top_level(u) && !u.key.starts_with("(anonymous)") && !u.key.starts_with("impl ")
        })
        .fold(
            HashMap::new(),
            |mut m: HashMap<String, Vec<(usize, usize)>>, u| {
                m.entry(u.key.clone())
                    .or_default()
                    .push((u.start_line, u.end_line));
                m
            },
        )
}

#[cfg(test)]
#[path = "../../tests/unit/fourclass/stacking.rs"]
mod tests;
