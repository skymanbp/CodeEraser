//! Locating and driving the `claude` CLI (Claude Code): on PATH as
//! npm's shim (`claude.cmd` on Windows, a script elsewhere) or the
//! native binary, else the native installer's `~/.local/bin/claude`.
//! Everything setup asks Claude Code goes through here — one spawn
//! throat, so a `.cmd` shim (which CreateProcess cannot run) rides
//! cmd.exe in exactly one place.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// PATH first (the shim or the native binary), then the native
/// installer's `~/.local/bin`; None = no Claude Code setup can see.
pub fn locate() -> Option<PathBuf> {
    let names: &[&str] = if cfg!(windows) {
        &["claude.exe", "claude.cmd", "claude.bat"]
    } else {
        &["claude"]
    };
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |n| dir.join(n)))
        .find(|p| p.is_file())
        .or_else(|| {
            let bin = home()?.join(".local").join("bin");
            names.iter().map(|n| bin.join(n)).find(|p| p.is_file())
        })
}

/// The user's home: `USERPROFILE` on Windows, `HOME` elsewhere.
pub fn home() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

/// Spawn claude with `args`. A `.cmd` / `.bat` shim is a script for
/// cmd.exe, reached through ComSpec so a scrubbed PATH cannot lose it.
pub fn run(exe: &Path, args: &[&str]) -> Result<Output> {
    let shim = exe
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"));
    let mut cmd = if shim {
        let mut c = Command::new(std::env::var_os("ComSpec").unwrap_or_else(|| "cmd.exe".into()));
        c.arg("/c").arg(exe);
        c
    } else {
        Command::new(exe)
    };
    cmd.args(args)
        .output()
        .with_context(|| format!("run {}", exe.display()))
}

/// Run and require success; the error names the claude command and
/// carries the first non-empty stderr line (the human's clue).
pub fn ok(exe: &Path, args: &[&str]) -> Result<()> {
    let out = run(exe, args)?;
    if out.status.success() {
        return Ok(());
    }
    let text = String::from_utf8_lossy(&out.stderr);
    let why = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("no stderr");
    bail!("claude {} exited {}: {why}", args.join(" "), code(&out))
}

fn code(out: &Output) -> String {
    out.status
        .code()
        .map_or_else(|| "?".to_string(), |c| c.to_string())
}

/// The configured marketplaces' names: `plugin marketplace list
/// --json`, and for a CLI too old for the flag, the prose listing.
pub fn marketplaces(exe: &Path) -> Result<Vec<String>> {
    let mut out = run(exe, &["plugin", "marketplace", "list", "--json"])?;
    if !out.status.success() {
        out = run(exe, &["plugin", "marketplace", "list"])?;
    }
    if !out.status.success() {
        bail!("claude plugin marketplace list exited {}", code(&out));
    }
    Ok(names_in(&String::from_utf8_lossy(&out.stdout)))
}

/// Names in a listing: the JSON array's `name` fields, or — for the
/// prose form — every row that is one bare word once its leading
/// marker glyph (`❯`, `>`, `*`, `-`) is stripped; `Source: …` rows
/// and headings carry a colon or a space and drop out.
pub fn names_in(listing: &str) -> Vec<String> {
    if let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(listing.trim()) {
        return rows
            .iter()
            .filter_map(|r| r["name"].as_str().map(str::to_string))
            .collect();
    }
    listing
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .trim()
        })
        .filter(|t| !t.is_empty() && !t.contains([' ', ':']))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/setup/claude.rs"]
mod tests;
