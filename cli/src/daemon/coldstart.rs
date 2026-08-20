//! Cold-start index state (ADR-003). A never-built index answers
//! probes with matches=0 — indistinguishable from a genuine clean
//! probe, which is the silent-failure class §5.9 forbids. So probes
//! answer an explicit Error (the client maps any non-report to
//! degraded and fails open) while the first build runs. UNKNOWN
//! (the init thread hasn't counted yet) is distinct from BUILDING.
//! Completeness is a POSITIVE cross-process fact: the meta
//! `full_build` stamp a finished analyze writes — a bare row count
//! was a per-process premise the v1.7 multi-writer amendment
//! retired (clearance review: an external `ce dedup` mid-run has
//! already committed SOME files, and "populated" read as
//! "complete").

use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};

const INDEX_UNKNOWN: u8 = 0;
const INDEX_BUILDING: u8 = 1;
const INDEX_READY: u8 = 2;
const INDEX_FAILED: u8 = 3;
static INDEX_STATE: AtomicU8 = AtomicU8::new(INDEX_UNKNOWN);

/// The positive completeness fact (dedup::analyze stamps it at the
/// end of every FULL pass) — never a row count.
fn full_build_done(root: &Path) -> bool {
    use crate::dedup::{Params, index::Index};
    Index::open(&root.join(".ce/index.db"), Params::default())
        .and_then(|idx| idx.full_build_done())
        .unwrap_or(false)
}

/// A completed full analyze IS a ready index, whatever the cold-start
/// thread is up to (CI caught the missing edge: Dedup-then-Probe
/// answered "cold start" on a fully built index).
pub fn mark_ready() {
    INDEX_STATE.store(INDEX_READY, Ordering::Release);
}

/// serve()-time init, entirely off the serve thread — even the stamp
/// check is a synchronous SQLite open, and any blocking before the
/// accept loop delays daemon readiness past the client's spawn-retry
/// window on a slow CI disk (a missed shutdown then leaks a 30-min
/// daemon that locks the exe). A stamped index is serveable as-is; an
/// unstamped one gets its ADR-003 first build. This build CAN
/// interleave with a Dedup request or an external `ce dedup` — benign
/// under the v1.7 convergent-cache contract, pinned by the
/// concurrent_writers battery's daemon leg. The failure transition
/// is a compare_exchange: a concurrent Dedup request may have built
/// the full index and marked READY — never clobber it.
pub fn init(root: &Path) {
    let root = root.to_path_buf();
    std::thread::spawn(move || {
        if full_build_done(&root) {
            mark_ready();
            return;
        }
        INDEX_STATE.store(INDEX_BUILDING, Ordering::Release);
        eprintln!("ce daemon: cold start, building first index");
        match crate::dedup::analyze(&root, None, None, None) {
            Ok(_) => mark_ready(),
            Err(e) => {
                eprintln!("ce daemon: first index build failed: {e:#}");
                let _ = INDEX_STATE.compare_exchange(
                    INDEX_BUILDING,
                    INDEX_FAILED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            }
        }
    });
}

/// READY is settled; BUILDING never short-circuits on a stamp check
/// (the stamp only lands at the END of a full pass — mid-build it is
/// absent, exactly the honest answer). UNKNOWN and FAILED consult
/// the stamp: a stamped index is a completed one — including FAILED
/// healed by an out-of-process build (Stop audit, hand-run
/// `ce dedup`) since.
pub fn index_ready(root: &Path) -> bool {
    match INDEX_STATE.load(Ordering::Acquire) {
        // READY is a CACHE of a cross-process fact, not the fact: the
        // db is wiped out of process on any schema/tokenizer/GRAPH_REV
        // change, so a pinned hook binary and a `cargo install`ed one
        // wipe each other on every run — and a stale READY answered
        // every probe matches=0, the silent-clean this module opens by
        // forbidding. Re-consult the stamp; a false one falls back.
        INDEX_READY if full_build_done(root) => true,
        INDEX_READY => {
            INDEX_STATE.store(INDEX_UNKNOWN, Ordering::Release);
            false
        }
        INDEX_BUILDING => false,
        _ => {
            let ready = full_build_done(root);
            if ready {
                mark_ready();
            }
            ready
        }
    }
}
