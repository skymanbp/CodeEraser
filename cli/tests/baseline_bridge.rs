//! The scalar↔set bridge (design §7.3 as amended 2026-08-14, the
//! conservation form): the dedup budget counts BLOCKS, the baseline
//! set holds unit-pair MEMBERS, and the two spaces are welded by one
//! exact accounting — members + collapsed == blocks == budget — so
//! neither gate can drift silently while the §7.2 identity keeps its
//! deliberate move-stability (57 of the self repo's 97 blocks share
//! a unit pair with an earlier block; that collapse is REPORTED).
//! Plus the ADR-006 independence pair: the --fail-under floor and
//! the ratchet each fail ALONE.

mod common;

use codeeraser::score::{self, Opts};
use std::path::{Path, PathBuf};

fn core_bin() -> String {
    std::env::var("CE_CORE_BIN").expect(
        "CE_CORE_BIN is unset — build the core and export it:\n  \
         cd core && cabal build all && export CE_CORE_BIN=$(cabal list-bin ce-core)",
    )
}

fn opts(db: Option<PathBuf>, floor: Option<u32>) -> Opts {
    Opts {
        db,
        core: core_bin(),
        days: None,
        floor,
        establish: false,
    }
}

/// Conservation on the self repo, every leg independently sourced:
/// blocks from the dedup report, budget from ce.toml, members and
/// collapse from the score assembly — and the two gates agree.
#[test]
fn bridge_conserves_blocks_into_members_and_gates_agree() {
    let root = Path::new("..");
    let db = common::tmp("bridge-db").join("index.db");
    let o = score::run(root, opts(Some(db.clone()), None)).expect("check");
    let (found, _s) = codeeraser::dedup::analyze(root, Some(db), None, None).expect("dedup");
    let budget = codeeraser::config::Config::load(root)
        .expect("ce.toml")
        .dedup
        .budget
        .expect("budget");
    assert_eq!(
        o.members + o.collapsed,
        found.blocks.len(),
        "every block lands in exactly one member (new or collapsed)"
    );
    assert_eq!(found.blocks.len(), budget, "the scalar gate's own equality");
    // the two gates judge the same repo the same way
    assert!(found.blocks.len() <= budget, "dedup --check passes");
    assert!(!o.reply.fail, "ce check passes: {:?}", o.reply.added);
    assert!(o.reply.degraded.is_none(), "healthy judgment");
}

/// The 3k corpus-generation gate (RM14, pre-registered): the frozen
/// pre-Haskell discrete member set must stay a SUBSET of every later
/// committed baseline — growth is a batch of named additions, never
/// a silent rewrite of history.
#[test]
fn pre_haskell_members_survive_every_generation() {
    let frozen: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("../contracts/eval/pre-haskell-members-v1.json").expect("frozen"),
    )
    .expect("frozen json");
    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("../ce-baseline.json").expect("committed baseline"),
    )
    .expect("baseline json");
    let now: std::collections::HashSet<u64> = baseline["discrete"]
        .as_array()
        .expect("discrete")
        .iter()
        .map(|v| v.as_u64().expect("member id"))
        .collect();
    let old: Vec<u64> = frozen["members"]
        .as_array()
        .expect("members")
        .iter()
        .map(|v| v.as_u64().expect("member id"))
        .collect();
    assert_eq!(old.len(), 40, "the frozen set is the 3j-close 40");
    for m in &old {
        assert!(
            now.contains(m),
            "pre-Haskell member {m} vanished from the committed baseline — \
             corpus growth must never rewrite the pre-generation set"
        );
    }
}

/// ADR-006: floor and ratchet each fail ALONE — the floor trips with
/// an empty added set, the ratchet trips with no floor on the wire.
#[test]
fn floor_and_ratchet_fail_independently() {
    let dir = common::tmp("bridge-both");
    common::seed_clone_pair(&dir);
    let est = score::run(&dir, opts(None, None)).expect("establish");
    assert!(!est.reply.fail, "no baseline, no floor: nothing fails");
    score::baseline::write(&dir, &est.reply.new_baseline).expect("write");

    // direction 1: floor alone (the ratchet half is clean)
    let floored = score::run(&dir, opts(None, Some(1000))).expect("floored");
    assert!(floored.reply.fail, "the floor alone must fail");
    assert!(
        floored.reply.added.is_empty() && floored.reply.over.is_empty(),
        "ratchet half stayed clean: {:?}",
        floored.reply.added
    );
    assert!(floored.reply.score < 1000, "the clone pair costs score");

    // direction 2: ratchet alone (no floor on the wire)
    std::fs::write(dir.join("c.rs"), common::rust_fn(3)).expect("c.rs");
    let grown = score::run(&dir, opts(None, None)).expect("grown");
    assert!(grown.reply.fail, "a new discrete member alone must fail");
    assert!(
        !grown.reply.added.is_empty(),
        "the new clone is a new member"
    );
}
