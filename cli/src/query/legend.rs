//! The query family's legend (plan v2.31 step 2, design booklet §4.2):
//! the fact schema the core judges by (CE.Query.Schema, mirrored row
//! for row — the core echoes it under `schema` and the unit leg holds
//! the two equal), the sort names, the enum vocabularies the fact
//! assembler spells into name hashes, and the hash itself. No name
//! crosses the wire (§5.9.2): a program's constants go as fnv1a64,
//! and every answer comes back as ids this side labels again through
//! the request's own tables.

use std::sync::OnceLock;

/// `code name sort…` per line, sorts by name — CE.Query.Schema's
/// rows in its order; the code IS the line's position.
const SCHEMA: &str = "\
0 node node sym
1 file node
2 in_dir node dir
3 dir dir
4 parent dir dir
5 dir_name dir sym
6 lang node sym
7 role node sym
8 lines node int
9 ref node node sym int
10 unresolved node int
11 unit unit node
12 unit_kind unit sym
13 unit_lines unit int
14 unit_at unit int
15 named unit sym
16 exported unit
17 coc unit int
18 cyclo unit int
19 nesting unit int
20 params unit int
21 clone unit unit sym
22 dup node node int
23 docdup node node
24 mention sym node
25 class node sym
26 set set node
";

/// The sorts by code (CE.Query.Cost: node 0 … set 5); the open sort
/// a position nothing constrained resolves to is −1 and reads `open`.
const SORT_NAMES: [&str; 6] = ["node", "dir", "unit", "int", "sym", "set"];

/// Program predicates number from here (CE.Query.Cost.idbFloor).
pub const IDB_FLOOR: u32 = 1000;

/// The set predicate — the `in(F, "glob")` sugar's target — and the
/// position its set sits in.
pub const SET_PRED: &str = "set";
pub const SET_ARG: usize = 0;

/// One schema predicate: its code, its name, the sort per position.
pub struct Pred {
    pub code: u32,
    pub name: &'static str,
    pub sorts: Vec<i64>,
}

fn table() -> &'static [Pred] {
    static T: OnceLock<Vec<Pred>> = OnceLock::new();
    T.get_or_init(|| {
        SCHEMA
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let mut w = line.split(' ');
                let code: u32 = w.next().and_then(|c| c.parse().ok()).expect("schema code");
                assert_eq!(code as usize, i, "schema codes are the line positions");
                let name = w.next().expect("schema name");
                let sorts = w.map(sort_code).collect();
                Pred { code, name, sorts }
            })
            .collect()
    })
}

/// The schema predicate of a name.
pub fn pred(name: &str) -> Option<&'static Pred> {
    table().iter().find(|p| p.name == name)
}

/// The schema predicate of a code.
pub fn pred_of(code: u32) -> Option<&'static Pred> {
    table().get(code as usize)
}

/// The schema as the core echoes it: `[code, arity, sorts…]` per row.
pub fn schema_rows() -> Vec<Vec<i64>> {
    table()
        .iter()
        .map(|p| {
            let mut row = vec![i64::from(p.code), p.sorts.len() as i64];
            row.extend(&p.sorts);
            row
        })
        .collect()
}

/// A sort's name; the open sort (and any code outside the table)
/// reads `open`.
pub fn sort_name(sort: i64) -> &'static str {
    usize::try_from(sort)
        .ok()
        .and_then(|i| SORT_NAMES.get(i))
        .copied()
        .unwrap_or("open")
}

/// A sort's code by name.
pub fn sort_code(name: &str) -> i64 {
    SORT_NAMES
        .iter()
        .position(|s| *s == name)
        .map(|i| i as i64)
        .unwrap_or_else(|| panic!("schema sort {name:?}"))
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
#[path = "../../tests/unit/query/legend.rs"]
mod tests;
