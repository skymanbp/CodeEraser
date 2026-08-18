//! candidates.rs battery, #[path]-mounted at the 300-line file gate
//! (the graph/md.rs precedent — and since GRAPH_REV 5 the ladder
//! reads the mount itself, so no entry_globs entry rides along).

use super::*;

fn unit(nodes: i64, hist: Vec<(u64, u32)>) -> Unit {
    Unit {
        path: "a.rs".into(),
        key: "k".into(),
        nth: 0,
        lang: "rs".into(),
        nodes,
        start_line: 1,
        end_line: 1,
        sig: vec![1],
        hist,
    }
}

/// The 0.85 boundary is admissible in BOTH directions: equality
/// survives (best case ted = max−min gives TSED exactly
/// min/max), provably-below is cut, and each bound owns its
/// tally. Row 3/4: sizes equal but the label multisets share
/// I = 84 (cut) vs 85 (kept) at max 100.
#[test]
fn prunes_sit_exactly_on_the_admissible_boundary() {
    let full = || vec![(1u64, 100u32)];
    let sharing = |c: u32| vec![(1, c), (2, 100 - c)];
    let table = [
        (85, full(), full(), (1, 0, 0)),
        (84, full(), full(), (0, 1, 0)),
        (100, sharing(84), full(), (0, 0, 1)),
        (100, sharing(85), full(), (1, 0, 0)),
    ];
    for (n1, h1, h2, want) in table {
        let units = vec![unit(n1, h1), unit(100, h2)];
        let union = BTreeMap::from([((0usize, 1usize), 2u8)]);
        let mut tally = Tally::default();
        let kept = prune(&units, union, &mut tally).len();
        assert_eq!(
            (kept, tally.pruned_size, tally.pruned_label),
            want,
            "n1 = {n1}"
        );
    }
}

/// S5's window IS the size bound, its label cut IS the shared
/// verdict, and an existing four-source pair is never duplicated
/// — the new pair carries bit 4 alone (frozen docs can never
/// hold it, so the source vocabulary stays epoch-safe).
#[test]
fn exhaustive_extension_windows_prunes_and_never_duplicates() {
    let full = || vec![(1u64, 100u32)];
    let mut c = Candidates {
        // ids 0..3: three ~100-node units sharing labels, one
        // 84-node unit outside the size window vs 100
        units: vec![
            unit(100, full()),
            unit(100, full()),
            unit(100, vec![(2, 100)]),
            unit(84, full()),
        ],
        pairs: vec![PairRow {
            a: 0,
            b: 1,
            sources: 0b0010,
        }],
        tally: Tally::default(),
    };
    extend_exhaustive(&mut c);
    let s5: Vec<(usize, usize, u8)> = c
        .pairs
        .iter()
        .filter(|p| p.sources == 1 << 4)
        .map(|p| (p.a, p.b, p.sources))
        .collect();
    // (0,1) stays four-source only; (0,2)/(1,2) are label-cut;
    // (3,*) sits outside the window on every side ≥100... except
    // 84·100 >= 85·100 is FALSE, so 84 pairs with nothing
    assert_eq!(s5, vec![], "no fresh pair survives here");
    assert_eq!(
        (c.tally.s5_already, c.tally.s5_pruned_label, c.tally.s5_new),
        (1, 2, 0)
    );
    // a genuinely similar unseen pair DOES land, bit 4 alone
    let mut c2 = Candidates {
        units: vec![unit(100, full()), unit(90, full())],
        pairs: vec![],
        tally: Tally::default(),
    };
    extend_exhaustive(&mut c2);
    assert_eq!(c2.tally.s5_new, 1);
    assert_eq!(
        (c2.pairs[0].a, c2.pairs[0].b, c2.pairs[0].sources),
        (0, 1, 1 << 4)
    );
}
