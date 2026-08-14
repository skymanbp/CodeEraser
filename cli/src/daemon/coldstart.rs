//! Cold-start index state (ADR-003). A never-built index answers
//! probes with matches=0 — indistinguishable from a genuine clean
//! probe, which is the silent-failure class §5.9 forbids. So probes
//! answer an explicit Error (the client maps any non-report to
//! degraded and fails open) while the first build runs. UNKNOWN
//! (the init thread hasn't counted yet) is distinct from BUILDING:
//! with no in-daemon analyze running, a populated index is a
//! complete one and may be counted inline; mid-build it would be a
//! partial index.

use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};

const INDEX_UNKNOWN: u8 = 0;
const INDEX_BUILDING: u8 = 1;
const INDEX_READY: u8 = 2;
const INDEX_FAILED: u8 = 3;
static INDEX_STATE: AtomicU8 = AtomicU8::new(INDEX_UNKNOWN);

fn indexed_file_count(root: &Path) -> i64 {
    use crate::dedup::{Params, index::Index};
    Index::open(&root.join(".ce/index.db"), Params::default())
        .and_then(|idx| idx.file_count())
        .unwrap_or(0)
}

/// A completed full analyze IS a ready index, whatever the cold-start
/// thread is up to (CI caught the missing edge: Dedup-then-Probe
/// answered "cold start" on a fully built index).
pub fn mark_ready() {
    INDEX_STATE.store(INDEX_READY, Ordering::Release);
}

/// serve()-time init, entirely off the serve thread — even the count
/// check is a synchronous SQLite open, and any blocking before the
/// accept loop delays daemon readiness past the client's spawn-retry
/// window on a slow CI disk (a missed shutdown then leaks a 30-min
/// daemon that locks the exe). An index with content is serveable
/// as-is; a never-built one gets its ADR-003 first build. This build
/// CAN interleave with a Dedup request or an external `ce dedup` —
/// benign under the v1.7 convergent-cache contract, pinned by the
/// concurrent_writers battery's daemon leg. The failure transition
/// is a compare_exchange: a concurrent Dedup request may have built
/// the full index and marked READY — never clobber it.
pub fn init(root: &Path) {
    let root = root.to_path_buf();
    std::thread::spawn(move || {
        if indexed_file_count(&root) > 0 {
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

/// READY is settled; BUILDING never short-circuits on a count check
/// (analyze commits per file — a mid-build count sees a partial
/// index, the blind window again, just shorter). UNKNOWN and FAILED
/// have no in-daemon analyze running, so the count is authoritative:
/// a populated index is a complete one — including FAILED healed by
/// an out-of-process build (Stop audit, hand-run `ce dedup`) since.
pub fn index_ready(root: &Path) -> bool {
    match INDEX_STATE.load(Ordering::Acquire) {
        INDEX_READY => true,
        INDEX_BUILDING => false,
        _ => {
            let ready = indexed_file_count(root) > 0;
            if ready {
                mark_ready();
            }
            ready
        }
    }
}
