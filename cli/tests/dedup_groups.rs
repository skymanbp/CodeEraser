//! Clone-group aggregation e2e (M2 attack-review R8): pairwise
//! blocks collapse into k-way families through the real analyze()
//! pipeline. Groups are the deflated denominator — blocks stay the
//! ratchet metric. Block/group counts are asserted inside
//! common::analyze (with the expected values passed per test).

mod common;

/// ~60 normalized tokens with a while/match skeleton — structurally
/// distinct from common::rust_fn's for/if shape, so the two families
/// can never cross-match into one component.
fn match_fn(seed: u32) -> String {
    format!(
        "fn pick_{seed}(k_{seed}: u64) -> u64 {{
    let mut acc_{seed} = {seed};
    while acc_{seed} < k_{seed} {{
        acc_{seed} = acc_{seed} * 3 + 1;
    }}
    match acc_{seed} % 4 {{
        0 => acc_{seed} + 11,
        1 => acc_{seed} * 2,
        2 => acc_{seed} / 5,
        _ => acc_{seed} - 7,
    }}
}}
"
    )
}

/// Three T2 copies across three files: C(3,2)=3 pairwise blocks must
/// aggregate into ONE family with three members.
#[test]
fn three_way_family_is_one_group() {
    let dir = common::tmp("groups-threeway");
    for (name, seed) in [("a.rs", 1), ("b.rs", 2), ("c.rs", 3)] {
        std::fs::write(dir.join(name), common::rust_fn(seed)).expect("seed");
    }
    let found = common::analyze(&dir, 3, 1);
    let g = &found.groups[0];
    let files: Vec<&str> = g.members.iter().map(|m| m.file.as_str()).collect();
    assert_eq!(files, ["a.rs", "b.rs", "c.rs"], "every copy is a member");
    assert_eq!(g.blocks, 3, "all pairwise blocks folded in");
    assert_eq!(g.tokens, found.blocks[0].tokens, "longest verified run");
}

/// Two structurally unrelated clone pairs: the components must NOT
/// fuse — two groups of two members each, ordered by first member.
#[test]
fn independent_families_stay_separate() {
    let dir = common::tmp("groups-independent");
    common::seed_clone_pair(&dir); // a.rs + b.rs (for/if family)
    std::fs::write(dir.join("c.rs"), match_fn(3)).expect("c.rs");
    std::fs::write(dir.join("d.rs"), match_fn(4)).expect("d.rs");
    let found = common::analyze(&dir, 2, 2);
    for g in &found.groups {
        assert_eq!(g.members.len(), 2);
        assert_eq!(g.blocks, 1);
    }
    let first: Vec<&str> = found.groups[0]
        .members
        .iter()
        .map(|m| m.file.as_str())
        .collect();
    assert_eq!(first, ["a.rs", "b.rs"], "groups ordered by first member");
}

/// Three T2 copies in ONE file yield two offset-class runs that share
/// the P1 span (dedup_pairs::three_copies_yield_offset_class_runs).
/// R8's core complaint — overlapping spans double-counted — must
/// collapse: one family, three disjoint members.
#[test]
fn same_file_overlapping_spans_merge() {
    let dir = common::tmp("groups-selfrepeat");
    let src = format!(
        "{}{}{}",
        common::rust_fn(1),
        common::rust_fn(2),
        common::rust_fn(3)
    );
    std::fs::write(dir.join("f.rs"), src).expect("f.rs");
    let found = common::analyze(&dir, 2, 1);
    let g = &found.groups[0];
    assert_eq!(g.members.len(), 3, "P1, P2, P3 — no double-count");
    assert_eq!(g.blocks, 2);
    for pair in g.members.windows(2) {
        assert!(pair[0].end <= pair[1].start, "members disjoint, ascending");
    }
}
