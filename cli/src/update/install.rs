//! Who owns the running binary — the fact that decides what
//! `ce update --yes` may touch. Detected from the executable's own
//! location, never from a config: the ledger a package manager keeps
//! is on disk beside the binary, and that is where it is read.

use std::path::{Path, PathBuf};

/// Frozen codes on the wire (ce.update-report `current.install`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Placed by hand (or by this updater): nothing else keeps a
    /// ledger of it, so the updater may replace it.
    Manual = 0,
    /// The installer bundle's sidecar: the GUI app beside it stays
    /// at its version until the new installer runs, and the
    /// document says so.
    Bundle = 1,
    /// `cargo install`'s bin dir — cargo's ledger, cargo's update.
    Cargo = 2,
    /// The plugin starter's bound copy — re-pinned by the plugin's
    /// own manifest, never overwritten under it.
    Plugin = 3,
}

pub struct Install {
    pub exe: PathBuf,
    pub kind: Kind,
}

impl Install {
    pub fn detect() -> Self {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("ce"));
        let data = std::env::var_os("CLAUDE_PLUGIN_DATA").map(PathBuf::from);
        let kind = classify(&exe, data.as_deref());
        Self { exe, kind }
    }
}

/// The classification, pure over paths (the e2e battery drives it
/// through env alone; the unit legs drive it through here).
pub fn classify(exe: &Path, plugin_data: Option<&Path>) -> Kind {
    let dir = exe.parent().unwrap_or(exe);
    if plugin_data.is_some_and(|d| same_dir(dir, d)) {
        return Kind::Plugin;
    }
    // ce.sh names its bound copies `ce-<version>-<key><ext>`: a
    // copy so named outside CLAUDE_PLUGIN_DATA is still the
    // starter's (a session without the env var set)
    let name = exe.file_name().map(|n| n.to_string_lossy().to_string());
    if name.is_some_and(|n| n.starts_with("ce-") && n.contains(env!("CARGO_PKG_VERSION"))) {
        return Kind::Plugin;
    }
    if dir.file_name().is_some_and(|n| n == "bin")
        && dir
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|n| n == ".cargo")
    {
        return Kind::Cargo;
    }
    let bundled = ["CodeEraser.exe", "CodeEraser", "codeeraser"];
    if bundled.iter().any(|b| dir.join(b).is_file()) {
        return Kind::Bundle;
    }
    Kind::Manual
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

/// One face's words for the install code — bilingual, CLI-side.
pub fn words(code: i64) -> String {
    crate::i18n::coded(
        code,
        &[
            ("placed by hand", "手工放置"),
            ("installer bundle sidecar", "安装包随附"),
            ("cargo install", "cargo 安装"),
            ("plugin starter's bound copy", "插件启动器绑定副本"),
        ],
        &[],
    )
}

#[cfg(test)]
#[path = "../../tests/unit/update/install.rs"]
mod tests;
