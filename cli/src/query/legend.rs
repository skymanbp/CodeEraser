//! The query family's legend (plan v2.31 step 2, design booklet §4.2):
//! the sort names an answer is labelled by, the enum vocabularies the
//! fact assembler spells into name hashes, and the hash itself. The
//! fact schema and its names are the core's (CE.Query.Schema; the
//! lexed program carries the names back, lexed.rs — plan v2.33 W1). No
//! name crosses the wire as a fact (§5.9.2): a constant goes as
//! fnv1a64, and every answer comes back as ids this side labels again
//! through the request's own tables.

/// The sorts by code (CE.Query.Cost: node 0 … set 5); the open sort
/// a position nothing constrained resolves to is −1 and reads `open`.
const SORT_NAMES: [&str; 6] = ["node", "dir", "unit", "int", "sym", "set"];

/// A sort's name; the open sort (and any code outside the table)
/// reads `open`.
pub fn sort_name(sort: i64) -> &'static str {
    named(&SORT_NAMES, sort).unwrap_or("open")
}

/// The one hash of a name — the same fnv1a64 the mention table and
/// the term bags key by, so `mention("foo", F)` meets the index's
/// own rows.
pub fn sym(name: &str) -> u64 {
    crate::dedup::tokens::fnv1a(name.as_bytes())
}

/// Node kinds: the graph wire's granularity codes, named by code
/// (graph::wire::GRAN_NAMES), then the two this family adds after
/// them — a walked asset (the wire's own bit) and a prose-only file
/// (the docdup family's universe, no graph node).
const GRANS: usize = crate::graph::wire::GRAN_NAMES.len();
pub const KIND_ASSET: usize = GRANS;
pub const KIND_PROSE: usize = GRANS + 1;
const ADDED_KINDS: [&str; 2] = ["asset", "prose"];

/// A node kind's name.
pub fn node_kind(kind: usize) -> &'static str {
    let grans = &crate::graph::wire::GRAN_NAMES;
    grans
        .iter()
        .chain(&ADDED_KINDS)
        .nth(kind)
        .copied()
        .unwrap_or("unknown")
}

/// The name at `code` in a table indexed by code (roles, references
/// and unit kinds are named where their codes are frozen: graph/
/// deadcode/flags.rs, graph/wire.rs, fourclass/kinds.rs).
pub fn named(table: &[&'static str], code: i64) -> Option<&'static str> {
    usize::try_from(code)
        .ok()
        .and_then(|i| table.get(i))
        .copied()
}

/// Clone kinds: a whole-unit twin inside a T1/T2 block, a T3 pair.
pub const CLONE_KINDS: [&str; 2] = ["t1t2", "t3"];

/// Every enum word the assembler can spell — the reverse dictionary
/// a `sym` answer reads its name back through.
pub fn vocabulary() -> impl Iterator<Item = &'static str> {
    (crate::graph::wire::GRAN_NAMES.iter())
        .chain(&ADDED_KINDS)
        .chain(&crate::graph::deadcode::ROLE_NAMES)
        .chain(&crate::graph::wire::EDGE_NAMES)
        .chain(&crate::fourclass::kinds::KIND_NAMES)
        .chain(&CLONE_KINDS)
        .copied()
}

#[cfg(test)]
#[path = "../../tests/unit/query/vocabulary.rs"]
mod tests;
