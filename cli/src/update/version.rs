//! Version and platform facts for the updater: the release version
//! grammar (`MAJOR.MINOR.PATCH`, the only shape this project tags)
//! and the release roster — the `<arch>-<os>` keys release.yml
//! builds, ce.sh derives from uname, and every pin, asset and
//! installer name spells (plan v2.29 step 10, O70: five since v1.7.0).

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

/// The release roster, in SHA256SUMS order: every `<arch>-<os>` key
/// the release matrix builds. ONE spelling — release.yml's matrix and
/// verify loop, ci.yml's rehearsal, scripts/roster.js and the
/// manifest's key set are held to it by tests/it/release_roster.rs —
/// and everything else about a target (binary suffix, bundle kind,
/// manifest key) derives from the key's `<os>` half below.
pub const TARGETS: [&str; 5] = [
    "x86_64-windows",
    "x86_64-linux",
    "aarch64-macos",
    "x86_64-macos",
    "aarch64-linux",
];

/// The first release whose matrix built every `TARGETS` row; a
/// manifest pinned before it carries the first three targets only
/// (the other two keys present but EMPTY — ce.sh's documented
/// air-gapped stance for them, PATH `ce` or a source install).
pub const FULL_ROSTER_SINCE: &str = "1.7.0";

/// The targets a manifest at `version` pins.
pub fn built(version: &str) -> &'static [&'static str] {
    if parse(version) >= parse(FULL_ROSTER_SINCE) {
        &TARGETS
    } else {
        &TARGETS[..3]
    }
}

/// The GUI bundle one target ships: NSIS on Windows, AppImage on
/// Linux, dmg on macOS — one kind per os, so the key decides.
#[derive(Clone, Copy)]
pub enum Bundle {
    Setup,
    AppImage,
    Dmg,
}

impl Bundle {
    /// (asset suffix after the key, manifest key tail).
    fn names(self) -> (&'static str, &'static str) {
        match self {
            Bundle::Setup => ("-setup.exe", "SETUP"),
            Bundle::AppImage => (".AppImage", "APPIMAGE"),
            Bundle::Dmg => (".dmg", "DMG"),
        }
    }

    pub fn suffix(self) -> &'static str {
        self.names().0
    }

    pub fn manifest_tail(self) -> &'static str {
        self.names().1
    }
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

    /// (os, arch) as std spells them → the roster key `<arch>-<os>`.
    /// A pair the roster never builds yields the literal
    /// `unsupported`, which no manifest carries a pin for — the check
    /// then reports the absent pin by name instead of inventing an
    /// asset URL.
    pub fn of(os: &str, arch: &str) -> Self {
        Self::of_key(&format!("{arch}-{os}"))
    }

    /// A roster key as written (`unsupported` for anything else).
    pub fn of_key(key: &str) -> Self {
        let key = TARGETS
            .iter()
            .copied()
            .find(|k| *k == key)
            .unwrap_or("unsupported");
        let ext = if os_of(key) == "windows" { ".exe" } else { "" };
        Self { key, ext }
    }

    /// The manifest's key spelling: `[a-z-]` uppercased to `[A-Z_]`
    /// (ce.sh `plat_key` → `CE_SHA256_<PLATFORM>_<ARTIFACT>`).
    pub fn manifest_key(&self) -> String {
        self.key.to_ascii_uppercase().replace('-', "_")
    }

    /// The GUI bundle this target ships, by its os half.
    pub fn bundle(&self) -> Option<Bundle> {
        match os_of(self.key) {
            "windows" => Some(Bundle::Setup),
            "linux" => Some(Bundle::AppImage),
            "macos" => Some(Bundle::Dmg),
            _ => None,
        }
    }

    /// The installer asset's name, by platform (release.yml).
    pub fn installer_asset(&self, version: &str) -> Option<String> {
        self.bundle()
            .map(|b| format!("CodeEraser-{version}-{}{}", self.key, b.suffix()))
    }
}

/// The `<os>` half of a roster key (`unsupported` has none).
fn os_of(key: &str) -> &str {
    key.rsplit_once('-').map_or("", |(_, os)| os)
}

#[cfg(test)]
#[path = "../../tests/unit/update/version.rs"]
mod tests;
