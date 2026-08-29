//! The updater's transport: curl, exactly as ce.sh drives it
//! (https only, redirects held to https, bounded time and size), and
//! `file://` as the hermetic test seam the same script uses. No HTTP
//! client is linked for this — the plugin already requires curl on
//! every host it downloads on, and one transport with one contract
//! beats two that agree most days.
//!
//! `CE_UPDATE_BASE` re-roots every endpoint under one prefix (the
//! e2e battery points it at a directory via `file://`): the release
//! index becomes `<base>/latest.json` and a tag's manifest
//! `<base>/<tag>/manifest.env`. Unset, the endpoints are GitHub's.

use anyhow::{Context, Result, bail};
use std::path::Path;

const REPO_API: &str = "https://api.github.com/repos/skymanbp/CodeEraser";
const REPO_RAW: &str = "https://raw.githubusercontent.com/skymanbp/CodeEraser";

fn base() -> Option<String> {
    std::env::var("CE_UPDATE_BASE")
        .ok()
        .filter(|b| !b.is_empty())
        .map(|b| b.trim_end_matches('/').to_string())
}

/// The newest published (non-draft, non-prerelease) release's tag.
pub fn latest_tag() -> Result<String> {
    let url = match base() {
        Some(b) => format!("{b}/latest.json"),
        None => format!("{REPO_API}/releases/latest"),
    };
    let text = text(&url)?;
    let v: serde_json::Value = serde_json::from_str(&text).context("release index is not JSON")?;
    let tag = v["tag_name"]
        .as_str()
        .filter(|t| !t.is_empty())
        .context("release index carries no tag_name")?;
    // the tag names a path segment below: only the grammar the
    // project tags with may ride into a URL
    if !tag
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        bail!("release tag {tag:?} is not a version tag");
    }
    Ok(tag.to_string())
}

/// The plugin manifest AS COMMITTED AT THE TAG — the pins the
/// release commit wrote, read from the repository, never from the
/// release's own mutable assets.
pub fn manifest_at(tag: &str) -> Result<String> {
    let url = match base() {
        Some(b) => format!("{b}/{tag}/manifest.env"),
        None => format!("{REPO_RAW}/{tag}/plugin/bin/manifest.env"),
    };
    text(&url)
}

/// A small text document (bounded at 1 MiB — a manifest is 3 KiB).
pub fn text(url: &str) -> Result<String> {
    if let Some(path) = url.strip_prefix("file://") {
        return std::fs::read_to_string(path).with_context(|| format!("read {path}"));
    }
    let out = curl(url, &["--max-time", "20", "--max-filesize", "1048576"])?;
    String::from_utf8(out).context("response is not UTF-8")
}

/// A binary asset to `dest`, bounded like ce.sh's fetch (120 s, 100
/// MiB). The caller verifies the bytes before anything reads them.
pub fn to_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(path) = url.strip_prefix("file://") {
        std::fs::copy(path, dest).with_context(|| format!("copy {path}"))?;
        return Ok(());
    }
    curl(
        url,
        &[
            "--max-time",
            "120",
            "--max-filesize",
            "104857600",
            "-o",
            &dest.display().to_string(),
        ],
    )
    .map(drop)
}

fn curl(url: &str, extra: &[&str]) -> Result<Vec<u8>> {
    if !url.starts_with("https://") {
        bail!("refusing non-https URL {url}");
    }
    let out = crate::proc::command("curl")
        .args(["-fsSL", "--proto", "=https", "--proto-redir", "=https"])
        .args(extra)
        .arg(url)
        .output()
        .context("spawn curl (the updater needs curl on PATH)")?;
    if !out.status.success() {
        bail!(
            "curl {url}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out.stdout)
}
