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
mod tests {
    use super::*;

    /// Re-establish replaces the previous token wholesale — the
    /// fresh-per-serve contract every battery leans on.
    #[test]
    fn establish_mints_hex_and_replaces_prior_token() {
        let root = crate::testutil::scratch("auth-mint");
        let first = establish(&root).expect("first");
        assert_eq!(first.len(), 64, "32 random bytes, hex");
        let second = establish(&root).expect("second");
        assert_ne!(first, second, "fresh per serve");
        assert_eq!(read(&root), second, "disk holds the latest");
    }

    /// A symlink squatting on the token path must never route the
    /// write: the daemon would truncate the link's TARGET with its
    /// own authority (review 2026-08-20 #2). The mint removes the
    /// link and writes a regular file; the victim keeps its bytes.
    #[cfg(unix)]
    #[test]
    fn establish_never_writes_through_a_symlink() {
        let root = crate::testutil::scratch("auth-symlink");
        let victim = root.join("victim.txt");
        std::fs::write(&victim, "precious").expect("victim");
        let token_path = root.join(TOKEN_FILE);
        std::fs::create_dir_all(token_path.parent().expect("parent")).expect("mkdir");
        std::os::unix::fs::symlink(&victim, &token_path).expect("plant link");
        establish(&root).expect("establish");
        assert_eq!(
            std::fs::read_to_string(&victim).expect("victim survives"),
            "precious"
        );
        let meta = std::fs::symlink_metadata(&token_path).expect("meta");
        assert!(meta.file_type().is_file(), "regular file, not a link");
    }

    /// A pre-existing wide-open token file must not keep its mode:
    /// truncate-in-place preserved 0644 (review 2026-08-20 #8); the
    /// remove + create-exclusive path is 0600 from the first byte.
    #[cfg(unix)]
    #[test]
    fn establish_tightens_a_preexisting_wide_mode() {
        use std::os::unix::fs::PermissionsExt;
        let root = crate::testutil::scratch("auth-mode");
        let token_path = root.join(TOKEN_FILE);
        std::fs::create_dir_all(token_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&token_path, "stale").expect("seed");
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o644))
            .expect("widen");
        establish(&root).expect("establish");
        let mode = std::fs::metadata(&token_path)
            .expect("meta")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "owner-only after re-mint");
    }
}
