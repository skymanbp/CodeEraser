//! Connection authorization for the project daemon. The socket name
//! is guessable (a hash of the project root), so the named pipe /
//! UDS accepts any local user — the capability that actually gates a
//! caller is READING <root>/.ce/daemon.token, minted fresh per
//! serve. That anchors daemon access to project-directory
//! readability, which is exactly the authority the daemon serves:
//! every reply is derived from the project's own content, so a user
//! the filesystem lets read the project learns nothing new here, and
//! one it refuses cannot probe, dedup-query, or shut the daemon
//! down. Unix tightens the file to 0600; Windows inherits the
//! project directory's ACL (the boundary itself).

use anyhow::{Context, Result};
use std::path::Path;

const TOKEN_FILE: &str = ".ce/daemon.token";

/// The refusal every unauthorized line gets — one string, so tests
/// and the client's retry match the same words.
pub const UNAUTHORIZED: &str = "unauthorized: hello with the token from .ce/daemon.token";

/// Mint and persist a fresh token for this serve. Called AFTER the
/// bind: the bind is the singleton race, and a loser writing first
/// would lock every client out of the daemon that actually won.
pub fn establish(root: &Path) -> Result<String> {
    let mut bytes = [0u8; 32];
    // map_err: getrandom's no-std Error lacks the std Error impl
    getrandom::fill(&mut bytes).map_err(|e| anyhow::anyhow!("token entropy: {e}"))?;
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let path = root.join(TOKEN_FILE);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("mkdir {}", dir.display()))?;
    }
    write_owner_only(&path, &token)?;
    Ok(token)
}

/// The client's read of the served token; empty when absent (no
/// daemon yet, or a pre-1.1.0 one) — the daemon answers with the
/// refusal that names the file.
pub fn read(root: &Path) -> String {
    std::fs::read_to_string(root.join(TOKEN_FILE))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Replace, never reuse: whatever sits at the path is REMOVED and
/// the token written through create-exclusive. Opening an existing
/// path would follow a symlink planted at this guessable location —
/// truncating the link's target with the daemon's authority — and
/// would keep a pre-existing file's wider mode instead of 0600
/// (review 2026-08-20 #2/#8); O_EXCL / CREATE_NEW does neither. On
/// Windows the mode line is moot — the file takes the project
/// directory's ACL, the boundary itself (module header).
fn write_owner_only(path: &Path, token: &str) -> Result<()> {
    use std::io::Write;
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).with_context(|| format!("clear stale {}", path.display())),
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600); // owner-only from the first byte
    }
    let mut f = opts
        .open(path)
        .with_context(|| format!("write {}", path.display()))?;
    f.write_all(token.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/daemon/auth.rs"]
mod tests;
