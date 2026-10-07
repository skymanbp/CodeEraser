//! The process's core session (plan v2.33 W2-text stage Z, item Z1): a
//! slot holding at most one healthy session per core between links (a
//! command names one core, so in practice one). `Link::open` takes the
//! session running the core asked for, and a link dropped healthy puts
//! its session back — so the families one command judges with (the
//! definition package, resolve/1, bags/1, graph/1 and the document, the
//! lockstep families, the audit, the advisor) ride one core process where
//! each opened its own before. Nothing is held across a request: two links
//! open at once (a family opening inside another's request, two threads)
//! are two sessions, and the slot keeps one of them; a session parked for
//! one core never ends another core's. Not every link
//! parks: the daemon keeps its own long-lived core and `ce doctor` asks a
//! fresh one (`Link::own`).

use super::session::Session;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

static SLOT: Mutex<Vec<Session>> = Mutex::new(Vec::new());

/// The core a session runs, as the slot compares it: the binary itself,
/// canonical, when it can be found — so `ce-core` resolved through
/// CE_CORE_BIN and the path the package loader located are one core —
/// else the name as given.
pub(super) fn key(core: &str) -> String {
    let effective = super::resolve_core(core);
    locate(&effective)
        .and_then(|p| std::fs::canonicalize(p).ok())
        .map_or(effective, |p| p.display().to_string())
}

/// The core binary itself: a path as given, a bare name on PATH.
pub fn locate(core: &str) -> Option<PathBuf> {
    let given = Path::new(core);
    if given.components().count() > 1 || given.is_absolute() {
        return given
            .is_file()
            .then(|| std::path::absolute(given).ok())
            .flatten();
    }
    let exe = format!("{core}.exe");
    let names: &[&str] = if cfg!(windows) {
        &[&exe, core]
    } else {
        &[core]
    };
    crate::proc::on_path(names)
}

/// The parked session when one runs `key`.
pub(super) fn take(key: &str) -> Option<Session> {
    let mut slot = SLOT.lock().unwrap_or_else(PoisonError::into_inner);
    let at = slot.iter().position(|parked| parked.key == key)?;
    Some(slot.swap_remove(at))
}

/// Park a healthy session; one parked before for the same core is ended
/// (outside the lock: ending a core waits for it).
pub(super) fn park(session: Session) {
    let before = {
        let mut slot = SLOT.lock().unwrap_or_else(PoisonError::into_inner);
        let at = slot.iter().position(|parked| parked.key == session.key);
        let before = at.map(|i| slot.swap_remove(i));
        slot.push(session);
        before
    };
    drop(before);
}

/// End every parked session: the CLI's last act, so a command's core
/// exits with it rather than at its pipe's EOF.
pub fn release() {
    let parked = std::mem::take(&mut *SLOT.lock().unwrap_or_else(PoisonError::into_inner));
    drop(parked);
}
