//! `ce update` — the product updating itself, ONE implementation
//! with three faces (CLI `ce update`, the GUI update screen, the
//! plugin's SessionStart notice + `/codeeraser:update`) and a fourth
//! read-only face on MCP (`update_check`). The check is a DOCUMENT
//! (ce.update-report), so every face renders one measurement.
//!
//! Trust anchor (ADR-007, and the reversal of frontier item O83 —
//! deferred while the anchor was assumed to be code signing): the
//! pins the RELEASE COMMIT wrote into `plugin/bin/manifest.env` at
//! the tag. The check reads the latest tag from GitHub, then that
//! tag's manifest from the repository at the tag, and the apply leg
//! places nothing whose SHA-256 the manifest does not spell. That is
//! the same chain ce.sh walks on every hook — a downloaded binary is
//! bytes the pin commit vouched for, or it is not placed.
//!
//! Ownership rule: an install a PACKAGE MANAGER owns is not
//! overwritten under it. The plugin's bound copy is re-pinned by the
//! plugin's own manifest (`/plugin update codeeraser`); a cargo
//! install is `cargo install codeeraser`. The apply leg replaces the
//! binaries only where nothing else keeps a ledger of them: a manual
//! placement, or the installer bundle's sidecars.

pub mod apply;
mod fetch;
pub mod install;
pub mod manifest;
pub mod notice;
pub mod version;

use serde_json::{Value, json};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.update-report/0.1.0";

/// The release page of a tag — the one URL a face may hand a human.
fn release_url(tag: &str) -> String {
    format!("https://github.com/skymanbp/CodeEraser/releases/tag/{tag}")
}

/// The check. Never returns an error: an unreachable network IS the
/// finding and rides inside the document as `latest.error`, with
/// `verdict` 2 (unknown). Every other field still answers, because
/// the running binary's own facts never needed the network.
pub fn document() -> Value {
    let plat = version::Platform::detect();
    let here = install::Install::detect();
    let latest = fetch::latest_tag();
    let (pins, pins_err) = match &latest {
        Ok(tag) => match fetch::manifest_at(tag).and_then(|t| manifest::pins(&t, &plat)) {
            Ok(p) => (Some(p), Value::Null),
            Err(e) => (None, json!(format!("{e:#}"))),
        },
        Err(_) => (None, Value::Null),
    };
    let verdict = verdict(&latest, pins.as_ref());
    json!({
        "schema": SCHEMA_ID,
        "current": {
            "version": env!("CARGO_PKG_VERSION"),
            "proto": crate::corelink::PROTO,
            "exe": here.exe.display().to_string(),
            // a CODE (plan v2.15: codes cross, each face owns its
            // sentence): 0 manual, 1 bundle, 2 cargo, 3 plugin
            "install": here.kind as u8,
        },
        "platform": {"key": plat.key, "ext": plat.ext},
        "latest": match &latest {
            Ok(tag) => json!({
                "tag": tag, "version": version::of_tag(tag),
                "url": release_url(tag), "error": Value::Null,
            }),
            Err(e) => json!({"tag": Value::Null, "version": Value::Null,
                             "url": Value::Null, "error": format!("{e:#}")}),
        },
        "pins": match &pins {
            Some(p) => p.json(),
            None => json!({"ce": Value::Null, "ceCore": Value::Null,
                           "installer": Value::Null, "baseUrl": Value::Null,
                           "error": pins_err}),
        },
        // 0 up to date, 1 update available, 2 unknown (the network
        // or the manifest did not answer — never read as "current")
        "verdict": verdict,
        // what `--yes` would do here: 0 nothing, 1 replace ce +
        // ce-core in place, 2 defer to `/plugin update codeeraser`,
        // 3 defer to `cargo install codeeraser`, 4 replace the
        // bundle's sidecars and save the installer for the GUI app
        "action": action(verdict, here.kind),
    })
}

/// The action codes whose apply leg is THIS process (the other two
/// name another package manager's command).
pub fn applies_here(action: u64) -> bool {
    matches!(action, 1 | 4)
}

fn verdict(latest: &anyhow::Result<String>, pins: Option<&manifest::Pins>) -> u8 {
    match (latest, pins) {
        (Ok(tag), Some(_)) => {
            let newer =
                version::parse(&version::of_tag(tag)) > version::parse(env!("CARGO_PKG_VERSION"));
            u8::from(newer)
        }
        // a tag with no readable manifest, or no tag at all: the
        // apply leg could place nothing, so the verdict must not
        // promise it
        _ => 2,
    }
}

fn action(verdict: u8, kind: install::Kind) -> u8 {
    if verdict != 1 {
        return 0;
    }
    match kind {
        install::Kind::Manual => 1,
        install::Kind::Plugin => 2,
        install::Kind::Cargo => 3,
        install::Kind::Bundle => 4,
    }
}

/// The console rendering — bilingual, from the DOCUMENT (the doctor
/// precedent: the renderer sits beside the measurement). Returns the
/// lines; the caller owns stdout and the exit code.
pub fn console(d: &Value) -> Vec<String> {
    use crate::i18n::{coded, line};
    let s = |p: &str, k: &str| d[p][k].as_str().unwrap_or("?").to_string();
    let verdict = d["verdict"].as_u64().unwrap_or(2) as i64;
    let mut out = vec![line(
        "ce {} (proto {}) at {} — {}",
        "ce {}（proto {}）位于 {} — {}",
        &[
            &s("current", "version"),
            &s("current", "proto"),
            &s("current", "exe"),
            &install::words(d["current"]["install"].as_u64().unwrap_or(0) as i64),
        ],
    )];
    // holes fill left to right (i18n::line): the unknown row's one
    // hole is the reason, the other rows' first hole the version
    let first = if verdict == 2 {
        d["latest"]["error"]
            .as_str()
            .or(d["pins"]["error"].as_str())
            .unwrap_or("?")
            .to_string()
    } else {
        s("latest", "version")
    };
    out.push(coded(
        verdict,
        &[
            ("latest: {} — up to date", "最新：{} — 已是最新"),
            (
                "latest: {} — update available ({})",
                "最新：{} — 有更新（{}）",
            ),
            ("latest: unknown — {}", "最新：未知 — {}"),
        ],
        &[&first, &s("latest", "url")],
    ));
    if verdict == 1 {
        out.push(action_words(d["action"].as_u64().unwrap_or(0) as i64));
    }
    out
}

/// The sentence for an `action` code — shared by the console face
/// and the SessionStart notice.
pub fn action_words(code: i64) -> String {
    crate::i18n::coded(
        code,
        &[
            ("nothing to do", "无需操作"),
            (
                "run `ce update --yes` to replace ce and ce-core in place (pins verified)",
                "运行 `ce update --yes` 就地替换 ce 与 ce-core（按 pin 校验）",
            ),
            (
                "this copy is the plugin's: run `/plugin update codeeraser` in Claude Code",
                "此副本属于插件：在 Claude Code 里运行 `/plugin update codeeraser`",
            ),
            (
                "this copy is cargo's: run `cargo install codeeraser`",
                "此副本由 cargo 管理：运行 `cargo install codeeraser`",
            ),
            (
                "run `ce update --yes --installer`: ce and ce-core are replaced beside the app, and the verified installer is saved for the GUI app itself",
                "运行 `ce update --yes --installer`：ce 与 ce-core 在应用旁就地替换，GUI 应用本体的安装包经校验后保存待运行",
            ),
        ],
        &[],
    )
}
