//! The judged∖U census (K40): the judged walk and the universe walk
//! contain neither the other, and every judged file the universe does
//! not hold is counted under the one cause that applies first. The
//! cause is decided with the universe's OWN rules (`walk::FILE_CAP`,
//! `walk::decode`, the nested-repository cut), so the census cannot
//! disagree with the walk it explains.

use super::walk;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::BTreeSet;
use std::path::Path;

/// Judged files the universe does not hold, by the causes the two
/// walks can differ on, in this precedence: the size cap, the pass's
/// binary rule (the judged walk reads such a file lossily), a nested
/// repository (cut whole from U, indexed by the judged walk), and
/// ignore semantics (the judged walk requires `.git` to honour
/// `.gitignore`; this one never does).
#[derive(Debug, Default, serde::Serialize)]
pub struct Outside {
    pub oversize: usize,
    pub binary: usize,
    pub nested: usize,
    pub ignored: usize,
}

/// The judged set (`files`) minus `live`, each member classified. A
/// file that vanished between the two reads is skipped — it is in
/// neither set now.
pub(super) fn outside(root: &Path, conn: &Connection, live: &BTreeSet<String>) -> Result<Outside> {
    let judged: Vec<String> = conn
        .prepare("SELECT path FROM files")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut out = Outside::default();
    for rel in judged.iter().filter(|p| !live.contains(*p)) {
        let path = root.join(rel);
        let Ok(meta) = std::fs::metadata(&path) else {
            continue;
        };
        if meta.len() > walk::FILE_CAP {
            out.oversize += 1;
        } else if std::fs::read(&path).is_ok_and(|b| walk::decode(&b).is_none()) {
            out.binary += 1;
        } else if walk::in_nested_repo(root, rel) {
            out.nested += 1;
        } else {
            out.ignored += 1;
        }
    }
    Ok(out)
}
