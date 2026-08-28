//! K30's code half (plan v2.17 L round piece (6)): the mounts tree
//! of graph_mounts.rs judged by the REAL core, every unmentioned
//! declaration's code read back through the names the wire carried.
//! Its own module beside the producer half because the two legs
//! share one fixture and one prelude (`graph_mounts::k30`) and the
//! producer file sits at its size line.

use crate::common;
use crate::graph_mounts::k30;
use codeeraser::graph::deadcode;
use std::collections::BTreeMap;

/// One line per row, `symbol code`: the six-cell mounts matrix (lib
/// root 0, private mount 1, bin root's `pub mod` 0 — the §6 residual,
/// pkg-private 1), the two collision cells (a private mount that is
/// also `pub(crate)` ⇒ 1, a `pub(crate)` in a re-exported file ⇒ 2),
/// the re-export target ⇒ 3, Go `package main` and `internal/` ⇒ 1
/// beside a library package ⇒ 0, a TS `export *` target ⇒ 3, and
/// cabal's exposed ⇒ 0 / other-modules ⇒ 1. `h` and `shown` are the
/// fixture's own unmentioned names (nothing spells them from another
/// file), so they are rows too.
const WANT_CODES: &str = "\
h 1
shown 1
l_unspoken 0
s_unspoken 3
sr_unspoken 2
m_unspoken 1
t_unspoken 1
sh_unspoken 0
Unspoken 1
XUnspoken 1
LibUnspoken 0
tsUnspoken 3
aUnspoken 0
bUnspoken 1
";

#[test]
fn every_k30_cell_reads_its_code_from_the_real_core() {
    let (_dir, _idx, w) = k30("graph-mounts-k30-codes", deadcode::Advisory::Yes);
    let reply = deadcode::judge(&common::core_bin(), &w, &[]).expect("judged");
    let rows: Vec<[i64; 4]> =
        serde_json::from_value(reply["exportUnmentioned"].clone()).expect("advisory rows");
    let names = w.unmentioned.as_ref().expect("names ride beside the keys");
    let mut got: BTreeMap<String, i64> = BTreeMap::new();
    for [node, vis, conv, code] in rows {
        for n in &names[&[node, vis, conv]] {
            assert!(
                got.insert(n.symbol.clone(), code).is_none(),
                "{} twice",
                n.symbol
            );
        }
    }
    let want: BTreeMap<String, i64> = WANT_CODES
        .lines()
        .map(|l| {
            let (s, c) = l.split_once(' ').expect("symbol code");
            (s.to_string(), c.parse().expect("code"))
        })
        .collect();
    assert_eq!(got, want);
}
