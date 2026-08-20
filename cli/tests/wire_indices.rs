//! Reply-side index boundary. The core's replies carry PAIR and DIR
//! indices, and this side turned them into slice subscripts without
//! checking: the request side has a machine-checked boundary contract
//! (Structure.hs `dirRow` refuses `d >= n`), the reply side had none.
//! That matters most in the daemon, which owns the fourclass link, has
//! no `catch_unwind` anywhere in the crate, and would take every
//! connected client down with one bad index (review 2026-08-19).

use codeeraser::fourclass::batch::{BatchClassification, Relocation};
use codeeraser::fourclass::session::{PathPair, report_json};
use codeeraser::fourclass::{ChangedLines, Classification, FourClass};

fn empty_pair() -> Classification {
    Classification {
        counts: FourClass::default(),
        moved: Vec::new(),
        relocated_units: Vec::new(),
        changed: ChangedLines {
            removed: Vec::new(),
            added: Vec::new(),
        },
        degraded: false,
    }
}

fn batch_with(relocations: Vec<Relocation>, suspicions: Vec<(usize, String)>) -> BatchClassification {
    BatchClassification {
        pairs: vec![empty_pair()],
        relocations,
        suspicions,
        degraded: None,
        link_failed: false,
    }
}

fn one(path: &str) -> Vec<PathPair> {
    vec![(Some(path.into()), Some(path.into()))]
}

/// In range, the name resolves — the assertion that keeps the
/// out-of-range case below from passing vacuously.
#[test]
fn a_pair_index_in_range_names_its_file() {
    let reloc = Relocation {
        from_pair: 0,
        from_unit: Some("work/1".into()),
        to_pair: 0,
        to_unit: Some("work/1".into()),
        lines: 3,
    };
    let v = report_json(&batch_with(vec![reloc], Vec::new()), &one("a.rs"));
    assert_eq!(v["relocations"][0]["from"], "a.rs");
    assert_eq!(v["relocations"][0]["to"], "a.rs");
}

/// Past the end, it reports null instead of panicking. `&pairs[idx]`
/// is what a mismatched or hostile ce-core reaches, and the daemon
/// carries no unwind guard.
#[test]
fn a_pair_index_past_the_end_is_null_not_a_panic() {
    let reloc = Relocation {
        from_pair: 7,
        from_unit: None,
        to_pair: 9,
        to_unit: None,
        lines: 1,
    };
    let batch = batch_with(vec![reloc], vec![(42, "stacking".into())]);
    let v = report_json(&batch, &one("a.rs"));
    assert!(v["relocations"][0]["from"].is_null(), "{v}");
    assert!(v["relocations"][0]["to"].is_null(), "{v}");
    assert!(v["suspicions"][0]["file"].is_null(), "{v}");
    assert_eq!(v["degraded"], serde_json::Value::Null);
}
