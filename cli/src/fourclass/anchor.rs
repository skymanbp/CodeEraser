//! The §7.2 container-chain identity of a unit (7.0.0, C-nth): what
//! tells two same-key units of one file apart across snapshots. Until
//! 7.0.0 the baseline used `nth`, the occurrence order by start line —
//! deleting an EARLIER sibling shifted every survivor's nth, and the
//! ratchet read one removal plus one addition for a clone nobody
//! touched; the churn ledger, the join and the seam pricer kept that
//! key until plan v2.30 step 5b item 29, so a survivor's ledger rows
//! from before the deletion and its HEAD identity disagreed and the
//! join found no row. The anchor is the CONTAINER CHAIN instead: the
//! keys of the units enclosing this one, outermost first, hashed —
//! `impl A { fn add }` and `impl B { fn add }` differ by their impl,
//! and deleting one leaves the other's anchor exactly where it was.
//! Every unit is anchored the same way, a top-level one included (the
//! empty chain hashes to one constant), so neither a body edit nor a
//! deletion elsewhere in the file ever moves an identity; two same-key
//! units under the SAME chain (a redefinition — a compile error in
//! Rust, the later wins in Python) are told apart by their order under
//! it, the one shape the old degradation survives, stated here — two
//! closures on ONE line are that shape too. A pure function of (key,
//! start, end) rows: the index's symbols table (score/anchor.rs) and a
//! fresh `units::segments` of any snapshot anchor alike, so the three
//! ledgers join exactly.

use crate::dedup::tokens::fnv1a;
use std::collections::HashMap;

/// One unit of one file with its §7.2 anchor.
pub struct Anchored {
    pub key: String,
    pub start: i64,
    pub end: i64,
    pub anchor: String,
}

/// The anchors of a segmented snapshot, one per unit in the units'
/// own order — the churn ledger, the join's HEAD table and the seam
/// pricer read a `units::segments` result through this one throat.
pub fn for_units(units: &[super::units::Unit]) -> Vec<String> {
    let spans = units
        .iter()
        .map(|u| (u.key.clone(), u.start_line as i64, u.end_line as i64))
        .collect();
    anchored(spans).into_iter().map(|a| a.anchor).collect()
}

/// Anchor one file's units (key, start, end): the hex fnv1a of the
/// container chain, then `#n` = the unit's start-line order among the
/// same-key units under that same chain (0 for every unit but a
/// redefinition's later copies). Pure and order-free — the input may
/// arrive in any order, the anchors depend on spans and keys alone.
pub fn anchored(units: Vec<(String, i64, i64)>) -> Vec<Anchored> {
    let chains: Vec<String> = units.iter().map(|u| chain_of(&units, u)).collect();
    let mut order: Vec<usize> = (0..units.len()).collect();
    order.sort_by_key(|&i| (units[i].1, units[i].2, units[i].0.as_str()));
    let mut groups: HashMap<(&str, &str), Vec<usize>> = HashMap::new();
    for &i in &order {
        groups
            .entry((units[i].0.as_str(), chains[i].as_str()))
            .or_default()
            .push(i);
    }
    let anchor = |i: usize| -> String {
        let group = &groups[&(units[i].0.as_str(), chains[i].as_str())];
        let n = group
            .iter()
            .position(|&j| j == i)
            .expect("every unit sits in its own group");
        format!("{:016x}#{n}", fnv1a(chains[i].as_bytes()))
    };
    units
        .iter()
        .enumerate()
        .map(|(i, (key, start, end))| Anchored {
            key: key.clone(),
            start: *start,
            end: *end,
            anchor: anchor(i),
        })
        .collect()
}

/// The keys of every unit STRICTLY enclosing `u`, outermost first,
/// NUL-joined — equal spans enclose nothing (two units on one line
/// are siblings, never each other's container).
fn chain_of(units: &[(String, i64, i64)], u: &(String, i64, i64)) -> String {
    let mut outer: Vec<&(String, i64, i64)> = units
        .iter()
        .filter(|c| c.1 <= u.1 && u.2 <= c.2 && (c.1, c.2) != (u.1, u.2))
        .collect();
    outer.sort_by_key(|c| std::cmp::Reverse(c.2 - c.1));
    outer
        .iter()
        .map(|c| c.0.as_str())
        .collect::<Vec<_>>()
        .join("\0")
}

#[cfg(test)]
#[path = "../../tests/unit/fourclass/anchor.rs"]
mod tests;
