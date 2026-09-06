//! The environment facts setup reads: who is running it versus who
//! is logged in (O73), and whether a directory is on PATH. The pure
//! halves (`same_user`, `dir_in`) take their inputs as arguments so
//! the unit legs drive them without touching the process environment.

use serde_json::{Value, json};
use std::ffi::OsStr;
use std::path::Path;

pub struct Users {
    pub process: String,
    pub logon: Option<String>,
    pub differs: bool,
}

impl Users {
    pub fn json(&self) -> Value {
        json!({"process": self.process, "logon": self.logon, "differs": self.differs})
    }
}

/// The running account and the interactive one. `differs` is true
/// only when BOTH are known and name different users: an unknown
/// logon (a service session, a CI runner) never refuses.
pub fn users() -> Users {
    let process = ["USERNAME", "USER", "LOGNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_else(|| "?".to_string());
    let logon = logon_user();
    let differs = logon.as_deref().is_some_and(|l| !same_user(l, &process));
    Users {
        process,
        logon,
        differs,
    }
}

/// The interactive logon: `CE_SETUP_LOGON_USER` (the battery's seam —
/// no test can log a second account in), `SUDO_USER` (sudo keeps the
/// invoking account there), else the console session's owner as
/// Windows reports it; None = unknown.
fn logon_user() -> Option<String> {
    ["CE_SETUP_LOGON_USER", "SUDO_USER"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .or_else(console_user)
}

/// `Win32_ComputerSystem.UserName` is the interactive console user
/// (`DOMAIN\name`) whatever token this process runs under — exactly
/// the pair an elevated installer has to compare.
#[cfg(windows)]
fn console_user() -> Option<String> {
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).UserName",
        ])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && !s.is_empty()).then_some(s)
}

#[cfg(not(windows))]
fn console_user() -> Option<String> {
    None
}

/// `DOMAIN\name` and `name` are the same account when the name parts
/// match; case-insensitively, because Windows account names are.
pub fn same_user(a: &str, b: &str) -> bool {
    let name = |s: &str| {
        s.rsplit(['\\', '/'])
            .next()
            .unwrap_or(s)
            .trim()
            .to_ascii_lowercase()
    };
    name(a) == name(b)
}

/// Whether `dir` is one of PATH's entries.
pub fn dir_on_path(dir: &Path) -> bool {
    std::env::var_os("PATH").is_some_and(|p| dir_in(dir, &p))
}

/// The pure half: canonicalised where the paths exist, trailing
/// separators dropped, case-insensitive on Windows (its filesystems
/// are), so `C:\Tools\` and `c:\tools` name one entry.
pub fn dir_in(dir: &Path, path: &OsStr) -> bool {
    let want = norm(dir);
    std::env::split_paths(path).any(|e| norm(&e) == want)
}

fn norm(p: &Path) -> String {
    let c = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let s = c
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_string();
    if cfg!(windows) {
        s.to_ascii_lowercase()
    } else {
        s
    }
}

#[cfg(test)]
#[path = "../../tests/unit/setup/env.rs"]
mod tests;
