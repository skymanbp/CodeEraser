//! The apply leg: download both binaries, verify BOTH against the
//! tag's pins, and only then place either — a pair, because a new
//! ce beside an old ce-core is a handshake refusal the user did not
//! ask for. Placement is two renames (target → `.old`, download →
//! target), which is what replaces a running executable on Windows
//! and is atomic per file everywhere; the `.old` twins are swept on
//! the next apply, when nothing runs them any more.

use super::manifest::Pins;
use super::version::Platform;
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Hex SHA-256 of a file — the pin's own spelling.
pub fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

/// Apply what the check document promised. `installer` also saves
/// the verified GUI installer beside the system temp dir and names
/// it — running an elevated installer is the user's click, never
/// this process's. Returns the report of what was placed.
pub fn run(doc: &Value, installer: bool) -> Result<Value> {
    match doc["verdict"].as_u64() {
        Some(1) => {}
        Some(0) => bail!("already up to date ({})", doc["current"]["version"]),
        _ => bail!(
            "cannot update: {}",
            doc["latest"]["error"]
                .as_str()
                .or(doc["pins"]["error"].as_str())
                .unwrap_or("the check did not conclude")
        ),
    }
    match doc["action"].as_u64() {
        Some(code) if super::applies_here(code) => {}
        Some(code) => bail!("{}", super::action_words(code as i64)),
        None => bail!("check document carries no action"),
    }
    let pins = pins_of(doc)?;
    let plat = Platform::of(std::env::consts::OS, std::env::consts::ARCH);
    let (ce_target, core_target) = targets(&plat);
    let dir = ce_target.parent().context("target has no directory")?;
    let swept = sweep_old(dir);
    let ce_tmp = fetch_verified(&pins, "ce", &pins.ce, &plat, &ce_target)?;
    let core_tmp = match fetch_verified(&pins, "ce-core", &pins.ce_core, &plat, &core_target) {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_file(&ce_tmp);
            return Err(e);
        }
    };
    place(&ce_tmp, &ce_target)?;
    place(&core_tmp, &core_target)?;
    let saved = if installer {
        Some(save_installer(&pins, &plat)?)
    } else {
        None
    };
    Ok(json!({
        "version": pins.version,
        "placed": [ce_target.display().to_string(), core_target.display().to_string()],
        "sweptOld": swept,
        "installer": saved.map(|p| p.display().to_string()),
    }))
}

fn pins_of(doc: &Value) -> Result<Pins> {
    let s = |v: &Value| v.as_str().map(str::to_string);
    let p = &doc["pins"];
    Ok(Pins {
        version: s(&doc["latest"]["version"]).context("no latest version")?,
        base_url: s(&p["baseUrl"]).context("no baseUrl pin")?,
        ce: s(&p["ce"]).context("no ce pin")?,
        ce_core: s(&p["ceCore"]).context("no ce-core pin")?,
        installer: s(&p["installer"]),
    })
}

/// Where the two binaries land: `ce` and `ce-core` BY NAME in the
/// running executable's directory — the sibling leg every resolver
/// in this product probes. By name, not the executable's own path:
/// inside the GUI the running executable is the app itself, and the
/// sidecars beside it are what this replaces. `CE_UPDATE_TARGET_DIR`
/// is the e2e seam (the battery must not replace the binary cargo
/// is running it from).
fn targets(plat: &Platform) -> (PathBuf, PathBuf) {
    let dir = std::env::var_os("CE_UPDATE_TARGET_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|e| e.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    (
        dir.join(format!("ce{}", plat.ext)),
        dir.join(format!("ce-core{}", plat.ext)),
    )
}

fn fetch_verified(
    pins: &Pins,
    name: &str,
    pin: &str,
    plat: &Platform,
    target: &Path,
) -> Result<PathBuf> {
    let url = pins.asset_url(name, plat);
    let tmp = target.with_extension(format!("download.{}", std::process::id()));
    if let Err(e) = super::fetch::to_file(&url, &tmp) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e).with_context(|| format!("download {url}"));
    }
    let got = sha256_hex(&tmp)?;
    if got != pin {
        let _ = std::fs::remove_file(&tmp);
        bail!("REFUSING {url}: SHA256 mismatch — expected {pin}, actual {got}");
    }
    Ok(tmp)
}

fn place(tmp: &Path, target: &Path) -> Result<()> {
    let old = target.with_extension("old");
    if target.exists() {
        std::fs::rename(target, &old).with_context(|| format!("retire {}", target.display()))?;
    }
    std::fs::rename(tmp, target).with_context(|| format!("place {}", target.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o755))?;
    }
    // the retired copy may still be the running process (Windows):
    // best effort now, swept for certain on the next apply
    let _ = std::fs::remove_file(&old);
    Ok(())
}

/// Remove the `.old` twins a previous apply could not delete while
/// they ran. Counted, so the report can say what it cleaned.
fn sweep_old(dir: &Path) -> usize {
    ["ce.old", "ce-core.old"]
        .iter()
        .map(|n| dir.join(n))
        .filter(|p| p.is_file() && std::fs::remove_file(p).is_ok())
        .count()
}

fn save_installer(pins: &Pins, plat: &Platform) -> Result<PathBuf> {
    let pin = pins
        .installer
        .as_deref()
        .context("this release carries no installer pin for this platform")?;
    let asset = plat
        .installer_asset(&pins.version)
        .context("no installer is built for this platform")?;
    let dir = std::env::temp_dir().join("codeeraser-update");
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(&asset);
    let url = format!("{}/{asset}", pins.base_url);
    super::fetch::to_file(&url, &dest).with_context(|| format!("download {url}"))?;
    let got = sha256_hex(&dest)?;
    if got != pin {
        let _ = std::fs::remove_file(&dest);
        bail!("REFUSING {url}: SHA256 mismatch — expected {pin}, actual {got}");
    }
    Ok(dest)
}
