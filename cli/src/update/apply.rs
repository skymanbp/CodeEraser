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
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Lowercase hexadecimal with two digits per byte, shared with receipts.
pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            write!(out, "{byte:02x}").expect("writing to a String");
            out
        })
}

/// Hex SHA-256 of a file — the pin's own spelling.
pub fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(hex(&Sha256::digest(&bytes)))
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
    place_pair((&ce_tmp, &ce_target), (&core_tmp, &core_target))?;
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

/// Place `ce`, then `ce-core`, as the PAIR the header promises (v2.35
/// G9, R3-39): when the second rename fails the first is rolled back,
/// so a failed apply never leaves a new `ce` beside an old core. The
/// retired copies are dropped only once both have landed.
pub(super) fn place_pair(ce: (&Path, &Path), core: (&Path, &Path)) -> Result<()> {
    let first = place(ce.0, ce.1)?;
    match place(core.0, core.1) {
        Ok(second) => {
            first.finish();
            second.finish();
            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(core.0);
            Err(match first.undo() {
                Ok(()) => e.context("ce-core was not placed; ce is back at the version it was"),
                Err(u) => e.context(format!(
                    "ce-core was not placed, and ce could not be put back ({u}) — re-run the update"
                )),
            })
        }
    }
}

/// One binary in place, its retired copy kept until the pair lands.
struct Placed {
    target: PathBuf,
    old: Option<PathBuf>,
}

impl Placed {
    /// Put the retired copy back (none: remove what was placed).
    fn undo(&self) -> std::io::Result<()> {
        std::fs::remove_file(&self.target)?;
        match &self.old {
            Some(old) => std::fs::rename(old, &self.target),
            None => Ok(()),
        }
    }

    /// The retired copy may still be the running process (Windows):
    /// best effort now, swept for certain on the next apply.
    fn finish(self) {
        if let Some(old) = self.old {
            let _ = std::fs::remove_file(old);
        }
    }
}

fn place(tmp: &Path, target: &Path) -> Result<Placed> {
    let old = target.with_extension("old");
    let old = if target.exists() {
        std::fs::rename(target, &old).with_context(|| format!("retire {}", target.display()))?;
        Some(old)
    } else {
        None
    };
    if let Err(e) = std::fs::rename(tmp, target) {
        if let Some(old) = &old {
            let _ = std::fs::rename(old, target);
        }
        return Err(e).with_context(|| format!("place {}", target.display()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(target, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(Placed {
        target: target.to_path_buf(),
        old,
    })
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

#[cfg(test)]
#[path = "../../tests/unit/update/apply.rs"]
mod tests;
