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

/// Both two-family layouts: `hub=false` puts each family in its own
/// file pair; `hub=true` co-locates one member of EACH family in
/// hub.rs with their spans touching on a shared source line.
fn seed_two_families(dir: &std::path::Path, hub: bool) {
    if hub {
        let joined = format!("{} {}", common::rust_fn(1).trim_end(), match_fn(9));
        std::fs::write(dir.join("hub.rs"), joined).expect("hub.rs");
        std::fs::write(dir.join("x.rs"), common::rust_fn(2)).expect("x.rs");
        std::fs::write(dir.join("y.rs"), match_fn(3)).expect("y.rs");
    } else {
        common::seed_clone_pair(dir); // a.rs + b.rs (for/if family)
        std::fs::write(dir.join("c.rs"), match_fn(3)).expect("c.rs");
        std::fs::write(dir.join("d.rs"), match_fn(4)).expect("d.rs");
    }
}

/// Two structurally unrelated clone pairs: the components must NOT
/// fuse — two groups of two members each, ordered by first member.
#[test]
fn independent_families_stay_separate() {
    let dir = common::tmp("groups-independent");
    seed_two_families(&dir, false);
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

/// Write `src` as a project's single f.rs and analyze, expecting one
/// group over `blocks` blocks (shared by the single-file cases).
fn single_file_family(name: &str, src: String, blocks: usize) -> codeeraser::dedup::pairs::Blocks {
    let dir = common::tmp(name);
    std::fs::write(dir.join("f.rs"), src).expect("f.rs");
    common::analyze(&dir, blocks, 1)
}

/// Three T2 copies in ONE file yield two offset-class runs that share
/// an IDENTICAL P1 anchor span (dedup_pairs::three_copies_yield_
/// offset_class_runs) — span interning fuses them: one family, three
/// members, no double-count of P1.
#[test]
fn same_file_offset_classes_share_identical_anchor() {
    let src = format!(
        "{}{}{}",
        common::rust_fn(1),
        common::rust_fn(2),
        common::rust_fn(3)
    );
    let found = single_file_family("groups-selfrepeat", src, 2);
    let g = &found.groups[0];
    assert_eq!(g.members.len(), 3, "P1, P2, P3 — no double-count");
    assert_eq!(g.blocks, 2);
    for pair in g.members.windows(2) {
        assert!(pair[0].end <= pair[1].start, "members disjoint, ascending");
    }
}

/// Attack-review regression (R8 batch): a file holding TWO unrelated
/// cloned regions whose spans TOUCH on a shared line must not bridge
/// the families — x.rs and y.rs share zero tokens and must never be
/// reported as mutual copies.
#[test]
fn hub_file_does_not_bridge_families() {
    let dir = common::tmp("groups-hub");
    seed_two_families(&dir, true);
    let found = common::analyze(&dir, 2, 2);
    for g in &found.groups {
        assert_eq!(g.members.len(), 2, "each family keeps its own pair");
    }
    // per-group token attribution: the two families differ in run
    // length, so a global max would show up as equal values here
    assert_ne!(
        found.groups[0].tokens, found.groups[1].tokens,
        "tokens is per-group, not a global max"
    );
}

/// Attack-review regression (R8 batch): a same-file adjacent pair
/// whose runs split one physical line (a_end == b_start) is TWO
/// occurrences — the group view must never collapse it to one member
/// and thereby deny the duplication its own block reports.
#[test]
fn adjacent_self_pair_keeps_two_members() {
    let src = format!("{} {}", common::rust_fn(1).trim_end(), common::rust_fn(2));
    let found = single_file_family("groups-adjacent", src, 1);
    let g = &found.groups[0];
    assert_eq!(g.members.len(), 2, "one block = two occurrences");
    assert_eq!(
        g.members[0].end, g.members[1].start,
        "the two runs share exactly the boundary line"
    );
}
