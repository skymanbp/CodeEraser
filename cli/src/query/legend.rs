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
    usize::try_from(sort)
        .ok()
        .and_then(|i| SORT_NAMES.get(i))
        .copied()
        .unwrap_or("open")
}

/// The one hash of a name — the same fnv1a64 the mention table and
/// the term bags key by, so `mention("foo", F)` meets the index's
/// own rows.
pub fn sym(name: &str) -> u64 {
    crate::dedup::tokens::fnv1a(name.as_bytes())
}

/// Node kinds, by the graph wire's granularity code, then the two
/// this family adds: a walked asset (the wire's own bit) and a
/// prose-only file (the docdup family's universe, no graph node).
pub const NODE_KINDS: [&str; 5] = ["file", "pkg", "section", "asset", "prose"];
pub const KIND_ASSET: usize = 3;
pub const KIND_PROSE: usize = 4;

/// Role names by role bit (graph/deadcode/flags.rs's positions).
pub const ROLE_NAMES: [&str; 10] = [
    "entry_named",
    "entry_dir",
    "test",
    "glob",
    "doc",
    "allow",
    "declared",
    "foreign",
    "unit",
    "asset",
];

/// Reference kinds by the graph wire's edge code (graph/wire.rs).
pub const REF_KINDS: [&str; 6] = [
    "import", "doc_link", "doc_ref", "asset", "contain", "refdef",
];

/// Unit kinds by fourclass::kinds code minus one (fn 1 … section 4).
pub const UNIT_KINDS: [&str; 4] = ["fn", "named", "impl", "section"];

/// Clone kinds: a whole-unit twin inside a T1/T2 block, a T3 pair.
pub const CLONE_KINDS: [&str; 2] = ["t1t2", "t3"];

/// Every enum word the assembler can spell — the reverse dictionary
/// a `sym` answer reads its name back through.
pub fn vocabulary() -> impl Iterator<Item = &'static str> {
    NODE_KINDS
        .iter()
        .chain(ROLE_NAMES.iter())
        .chain(REF_KINDS.iter())
        .chain(UNIT_KINDS.iter())
        .chain(CLONE_KINDS.iter())
        .copied()
}

#[cfg(test)]
#[path = "../../tests/unit/query/vocabulary.rs"]
mod tests;
