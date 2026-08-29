//! Version and platform facts for the updater: the release version
//! grammar (`MAJOR.MINOR.PATCH`, the only shape this project tags)
//! and the platform key grammar release.yml and ce.sh share.

/// A release version as three numbers; anything the grammar does
/// not cover parses as `None` and is never "newer" than anything.
pub fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.trim().split('.').map(|p| p.parse::<u64>().ok());
    let out = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(out)
}

/// `v1.3.0` → `1.3.0`; a tag without the prefix is its own version.
pub fn of_tag(tag: &str) -> String {
    tag.strip_prefix('v').unwrap_or(tag).to_string()
}

/// The platform key release.yml stages assets under and ce.sh
/// derives from uname — one grammar, spelled here for the Rust face.
pub struct Platform {
    pub key: &'static str,
    pub ext: &'static str,
}

impl Platform {
    pub fn detect() -> Self {
        Self::of(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// (os, arch) as std spells them → the release key. A pair the
    /// release matrix never builds yields the literal `unsupported`,
    /// which no manifest carries a pin for — the check then reports
    /// the absent pin by name instead of inventing an asset URL.
    pub fn of(os: &str, arch: &str) -> Self {
        let (key, ext) = match (os, arch) {
            ("windows", "x86_64") => ("x86_64-windows", ".exe"),
            ("linux", "x86_64") => ("x86_64-linux", ""),
            ("linux", "aarch64") => ("aarch64-linux", ""),
            ("macos", "aarch64") => ("aarch64-macos", ""),
            ("macos", "x86_64") => ("x86_64-macos", ""),
            _ => ("unsupported", ""),
        };
        Self { key, ext }
    }

    /// The manifest's key spelling: `[a-z-]` uppercased to `[A-Z_]`
    /// (ce.sh `plat_key` → `CE_SHA256_<PLATFORM>_<ARTIFACT>`).
    pub fn manifest_key(&self) -> String {
        self.key.to_ascii_uppercase().replace('-', "_")
    }

    /// The installer asset's name, by platform (release.yml).
    pub fn installer_asset(&self, version: &str) -> Option<String> {
        let suffix = match self.key {
            "x86_64-windows" => "-setup.exe",
            "x86_64-linux" => ".AppImage",
            "aarch64-macos" => ".dmg",
            _ => return None,
        };
        Some(format!("CodeEraser-{version}-{}{suffix}", self.key))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/update/version.rs"]
mod tests;
