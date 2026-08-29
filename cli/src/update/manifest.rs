//! The plugin manifest as the updater reads it: `plugin/bin/
//! manifest.env` is plain `KEY="value"` lines sourced by sh, and the
//! pins it carries are the release commit's own measurement of the
//! assets (docs/RELEASE.md §2). This reader accepts exactly the
//! shape the file is written in — a key, `=`, an optionally quoted
//! value — and nothing sh would evaluate: no expansion, no command.

use super::version::Platform;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Every `KEY=value` assignment, comments and blank lines dropped,
/// surrounding double quotes stripped once.
pub fn parse(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| {
            let v = v.trim();
            let v = v
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(v);
            (k.trim().to_string(), v.to_string())
        })
        .collect()
}

/// The three pins one platform needs, plus where the assets live.
#[derive(Debug)]
pub struct Pins {
    pub version: String,
    pub base_url: String,
    pub ce: String,
    pub ce_core: String,
    /// The GUI installer's pin — absent on a platform whose bundle
    /// the release never built.
    pub installer: Option<String>,
}

impl Pins {
    pub fn json(&self) -> Value {
        json!({
            "ce": self.ce, "ceCore": self.ce_core, "installer": self.installer,
            "baseUrl": self.base_url, "error": Value::Null,
        })
    }

    /// The asset URL of one binary at this manifest's version.
    pub fn asset_url(&self, name: &str, plat: &Platform) -> String {
        format!(
            "{}/{name}-{}-{}{}",
            self.base_url, self.version, plat.key, plat.ext
        )
    }
}

/// Read this platform's pins out of a manifest text. An EMPTY pin is
/// the manifest's documented air-gapped stance (ce.sh never
/// downloads on it) and is refused here by name for the same
/// reason: nothing vouches for the bytes.
pub fn pins(text: &str, plat: &Platform) -> Result<Pins> {
    let m = parse(text);
    let get = |k: &str| -> Result<String> {
        m.get(k)
            .filter(|v| !v.is_empty())
            .cloned()
            .with_context(|| format!("manifest carries no {k}"))
    };
    let plat_key = plat.manifest_key();
    let installer_key = match plat.key {
        "x86_64-windows" => "SETUP",
        "x86_64-linux" => "APPIMAGE",
        "aarch64-macos" => "DMG",
        _ => "",
    };
    Ok(Pins {
        version: get("CE_MANIFEST_VERSION")?,
        base_url: get("CE_BASE_URL")?,
        ce: get(&format!("CE_SHA256_{plat_key}_CE"))?,
        ce_core: get(&format!("CE_SHA256_{plat_key}_CECORE"))?,
        installer: get(&format!("CE_SHA256_{plat_key}_{installer_key}")).ok(),
    })
}

#[cfg(test)]
#[path = "../../tests/unit/update/manifest.rs"]
mod tests;
