//! The SessionStart notice: the check document, cached for a day
//! and rendered as one bracketed line when — and only when — a
//! newer release exists. Fail-open like every hook: no network, no
//! cache dir, a garbled cache — no line. `CE_UPDATE_CHECK=0` turns
//! the notice off; the test harness sets it so no battery reaches
//! the network by accident.

use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Where the cached document lives: the plugin's data dir when the
/// starter set one (the machine-wide place the plugin already owns),
/// the OS temp dir otherwise. Never under the project — the answer
/// is about this binary, not that tree.
fn cache_path() -> PathBuf {
    std::env::var_os("CLAUDE_PLUGIN_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("codeeraser-update-check.json")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The cached document if it is fresh AND was measured by this very
/// version — a cache written by the binary this one replaced would
/// keep announcing the update that already happened.
fn fresh_cached() -> Option<Value> {
    let text = std::fs::read_to_string(cache_path()).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let age = now_secs().saturating_sub(v["checkedSecs"].as_u64()?);
    let same_binary = v["document"]["current"]["version"] == env!("CARGO_PKG_VERSION");
    (age < TTL.as_secs() && same_binary).then(|| v["document"].clone())
}

fn store(doc: &Value) {
    let wrapped = serde_json::json!({"checkedSecs": now_secs(), "document": doc});
    let path = cache_path();
    // tmp + rename: two sessions share one data dir (ce.sh's stamp
    // takes the same care), and a half-written cache reads as none
    let tmp = path.with_extension(format!("json.{}", std::process::id()));
    if std::fs::write(&tmp, wrapped.to_string()).is_ok() && std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// The notice line, or nothing. An `unknown` verdict is cached too:
/// a host with no network must not pay the probe on every session.
pub fn session_notice() -> Option<String> {
    if std::env::var("CE_UPDATE_CHECK").is_ok_and(|v| v == "0") {
        return None;
    }
    let doc = fresh_cached().unwrap_or_else(|| {
        let d = super::document();
        store(&d);
        d
    });
    if doc["verdict"] != 1 {
        return None;
    }
    Some(crate::i18n::line(
        "[ce update: {} available — {}]",
        "〔ce 更新：{} 可用 — {}〕",
        &[
            &doc["latest"]["version"].as_str().unwrap_or("?"),
            &super::action_words(doc["action"].as_i64().unwrap_or(0)),
        ],
    ))
}
