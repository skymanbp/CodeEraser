//! Plan §7.3 property: after ANY sequence of upserts/deletes, the
//! incrementally-maintained index equals a from-scratch rebuild of
//! the final tree. Deterministic seed cases live in dedup_index.rs;
//! this randomizes the op sequence.

use codeeraser::dedup::{Params, index::Index};
use codeeraser::scan::lang::Lang;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common;
use crate::common::rust_fn;

static CASE: AtomicUsize = AtomicUsize::new(0);

fn case_dir() -> PathBuf {
    let n = CASE.fetch_add(1, Ordering::Relaxed);
    common::tmp(&format!("prop-{n}"))
}

#[derive(Debug, Clone)]
enum Op {
    Upsert(usize, Vec<u32>),
    Delete(usize),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..4usize, prop::collection::vec(0..8u32, 1..4))
            .prop_map(|(f, seeds)| Op::Upsert(f, seeds)),
        (0..4usize).prop_map(Op::Delete),
    ]
}

fn file_name(i: usize) -> String {
    format!("f{i}.rs")
}

fn content(seeds: &[u32]) -> String {
    seeds.iter().map(|s| rust_fn(*s)).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]
    #[test]
    fn incremental_equals_full_random(ops in prop::collection::vec(op_strategy(), 1..12)) {
        let dir = case_dir();
        let p = Params::default();
        let mut model: BTreeMap<String, String> = BTreeMap::new();
        let mut incr = Index::open(&dir.join("incr.db"), p).expect("open incr");
        for op in &ops {
            match op {
                Op::Upsert(i, seeds) => {
                    let (name, src) = (file_name(*i), content(seeds));
                    incr.refresh_file(&name, src.as_bytes(), Lang::Rust, p).expect("refresh");
                    model.insert(name, src);
                }
                Op::Delete(i) => {
                    model.remove(&file_name(*i));
                }
            }
            // reap after every op so deletions interleave with upserts
            let seen = incr.indexed_paths().expect("snapshot");
            incr.remove_missing(&model.keys().cloned().collect(), &seen)
                .expect("reap");
        }
        let mut full = Index::open(&dir.join("full.db"), p).expect("open full");
        for (name, src) in &model {
            full.refresh_file(name, src.as_bytes(), Lang::Rust, p).expect("rebuild");
        }
        prop_assert_eq!(
            incr.all_instances().expect("incr"),
            full.all_instances().expect("full")
        );
    }
}
