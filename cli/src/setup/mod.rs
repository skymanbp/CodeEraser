//! `ce setup` — one install = the whole product, on every platform
//! (plan v2.29 step 10: O72 / O73 / O82). Since v1.0.1 the Windows
//! installer's POSTINSTALL hook carried the Claude Code wiring as an
//! inline PowerShell program, so AppImage and dmg users had neither
//! the wiring nor a PATH entry. The wiring is ONE Rust body now: the
//! NSIS hook calls it and everyone else runs it once — locate Claude
//! Code, register the public marketplace at its `release` ref (which
//! verify-publish fast-forwards to every published tag, so an install
//! follows releases, not main), install or refresh the plugin, write
//! the marker the uninstaller keys on, and say whether this binary's
//! directory is on PATH. Every failure DEGRADES into the exit-code
//! legend the installer log has printed since v1.0.1 — wiring never
//! fails an install — and `--unwire` removes exactly what `setup`
//! added, keyed on the marker, never a registration a human made.
//!
//! O73: an elevated installer runs as the elevating account. Where
//! that is not the logged-in user, the plugin would land in the wrong
//! `~/.claude`; setup reads the pair and refuses by name (exit 13)
//! instead of wiring the wrong home.

pub mod claude;
pub mod env;

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// JSON output schema id; bump on shape change (plan §7.1).
pub const SCHEMA_ID: &str = "ce.setup-report/0.1.0";

/// The names `ce setup` spells — one table, which the NSIS hook, both
/// READMEs and `cli/tests/gui/installer_wiring.js` read off this
/// source rather than carrying copies (the 9f86d58 lesson: a stale
/// copy of the marketplace path dropped the guard for three days).
pub struct Names {
    /// The marketplace source: this repository at its `release` branch.
    pub source: &'static str,
    /// The marketplace's name — the root manifest's `name`.
    pub marketplace: &'static str,
    /// `<plugin>@<marketplace>`, what `claude plugin install` takes.
    pub plugin: &'static str,
    /// The file written beside the binary when THIS run registered the
    /// marketplace; `--unwire` and the NSIS PREUNINSTALL hook key on it.
    pub marker: &'static str,
}

pub const NAMES: Names = Names {
    source: "skymanbp/CodeEraser@release",
    marketplace: "codeeraser",
    plugin: "codeeraser@codeeraser",
    marker: "claude-plugin-wired",
};

/// The exit codes — the installer log's legend since v1.0.1, kept.
/// Compared and printed by `code()` alone, so no equality derive.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Exit {
    Wired = 0,
    Kept = 5,
    NoClaude = 10,
    AddFailed = 11,
    InstallFailed = 12,
    ElevatedUserDiffers = 13,
}

impl Exit {
    pub fn code(self) -> u8 {
        self as u8
    }
}

pub struct Opts {
    /// Remove what a previous `setup` added (marker-keyed).
    pub unwire: bool,
    /// Where the marker lives (default: this binary's directory —
    /// `$INSTDIR` for the installer).
    pub marker_dir: Option<PathBuf>,
}

/// The act as a DOCUMENT — every fact read and every step's state,
/// the exit code inside it — beside that code as a value, so the
/// console and the JSON face render one measurement (the update
/// precedent) and the process exits with what the document says.
pub struct Report {
    pub doc: Value,
    pub exit: Exit,
}

pub fn run(opts: &Opts) -> Report {
    let dir = opts.marker_dir.clone().unwrap_or_else(exe_dir);
    let marker = dir.join(NAMES.marker);
    let users = env::users();
    let mut doc = json!({
        "schema": SCHEMA_ID,
        "act": if opts.unwire { "unwire" } else { "wire" },
        "claude": Value::Null,
        "user": users.json(),
        "marketplace": {"source": NAMES.source, "name": NAMES.marketplace, "state": "skipped"},
        "plugin": {"target": NAMES.plugin, "state": "skipped"},
        "marker": {"path": marker.display().to_string(), "before": marker.is_file(), "after": marker.is_file()},
        "path": {"dir": dir.display().to_string(), "onPath": env::dir_on_path(&dir)},
        "exit": Exit::Wired.code(),
        "error": Value::Null,
    });
    let exit = if users.differs {
        Exit::ElevatedUserDiffers
    } else if let Some(exe) = claude::locate() {
        doc["claude"] = exe.display().to_string().into();
        if opts.unwire {
            unwire(&exe, &marker, &mut doc)
        } else {
            wire(&exe, &marker, &mut doc)
        }
    } else {
        Exit::NoClaude
    };
    doc["exit"] = exit.code().into();
    Report { doc, exit }
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Register (if absent), install, refresh, mark. A refused step ends
/// the act with its code and its reason in `error`; a registration
/// already there is kept and refreshed (5), never replaced — a dev
/// checkout registered as a directory is a human's, not ours.
fn wire(exe: &Path, marker: &Path, doc: &mut Value) -> Exit {
    let listed = match claude::marketplaces(exe) {
        Ok(names) => names,
        Err(e) => return refused(doc, "marketplace", Exit::AddFailed, e),
    };
    let fresh = !listed.iter().any(|n| n == NAMES.marketplace);
    if fresh && let Err(e) = claude::ok(exe, &["plugin", "marketplace", "add", NAMES.source]) {
        return refused(doc, "marketplace", Exit::AddFailed, e);
    }
    doc["marketplace"]["state"] = if fresh { "added" } else { "present" }.into();
    if let Err(e) = claude::ok(exe, &["plugin", "install", NAMES.plugin]) {
        return refused(doc, "plugin", Exit::InstallFailed, e);
    }
    // an installed copy is re-pinned to the marketplace's current
    // manifest; best effort — the install above already answered
    let _ = claude::ok(exe, &["plugin", "update", NAMES.plugin]);
    doc["plugin"]["state"] = "installed".into();
    if !fresh {
        return Exit::Kept;
    }
    match std::fs::write(marker, b"") {
        Ok(()) => doc["marker"]["after"] = true.into(),
        Err(e) => doc["error"] = format!("marker not written: {e}").into(),
    }
    Exit::Wired
}

/// Undo a wiring THIS binary did: no marker, nothing to remove (a
/// registration the human made stays theirs). The marker goes either
/// way — the installer is about to delete the directory it sits in.
fn unwire(exe: &Path, marker: &Path, doc: &mut Value) -> Exit {
    if !marker.is_file() {
        return Exit::Wired;
    }
    let plugin = claude::ok(exe, &["plugin", "uninstall", NAMES.plugin]);
    let market = claude::ok(exe, &["plugin", "marketplace", "remove", NAMES.marketplace]);
    doc["plugin"]["state"] = if plugin.is_ok() {
        "uninstalled"
    } else {
        "failed"
    }
    .into();
    doc["marketplace"]["state"] = if market.is_ok() { "removed" } else { "failed" }.into();
    let _ = std::fs::remove_file(marker);
    doc["marker"]["after"] = marker.is_file().into();
    match plugin.and(market) {
        Ok(()) => Exit::Wired,
        Err(e) => refused(doc, "unwire", Exit::InstallFailed, e),
    }
}

fn refused(doc: &mut Value, step: &str, code: Exit, err: anyhow::Error) -> Exit {
    if doc[step].is_object() {
        doc[step]["state"] = "failed".into();
    }
    doc["error"] = format!("{err:#}").into();
    code
}

/// The console rendering — bilingual, from the REPORT (the update
/// precedent: the renderer sits beside the measurement). Returns the
/// lines; the caller owns stdout and the exit code.
pub fn console(r: &Report) -> Vec<String> {
    use crate::i18n::line;
    let mut out = vec![verdict_line(r)];
    if r.doc["act"] == "wire" && r.doc["path"]["onPath"] == false {
        let dir = r.doc["path"]["dir"].as_str().unwrap_or("?");
        let hint = if cfg!(windows) {
            line(
                "add it to your PATH under System Properties → Environment Variables (the installer does this)",
                "在「系统属性 → 环境变量」里把它加进 PATH（安装包会代做）",
                &[],
            )
        } else {
            format!("export PATH=\"{dir}:$PATH\"")
        };
        out.push(line(
            "{} is not on PATH — {}",
            "{} 不在 PATH 上——{}",
            &[&dir, &hint],
        ));
    }
    out
}

fn verdict_line(r: &Report) -> String {
    use crate::i18n::line;
    let d = &r.doc;
    let s = |k: &str| d[k].as_str().unwrap_or("?").to_string();
    let u = |k: &str| d["user"][k].as_str().unwrap_or("?").to_string();
    let err = s("error");
    let (source, target, market) = (NAMES.source, NAMES.plugin, NAMES.marketplace);
    match (d["act"] == "unwire", r.exit) {
        (_, Exit::ElevatedUserDiffers) => line(
            "setup runs as {} but the logged-in user is {} — the plugin would land in the wrong home; run `ce setup` from your own account, unelevated",
            "setup 以 {} 运行，而登录用户是 {}——插件会装进错误的账户；请以你自己的账户、不提权地运行 `ce setup`",
            &[&u("process"), &u("logon")],
        ),
        (_, Exit::NoClaude) => line(
            "Claude Code not detected — plugin not wired; after installing Claude Code run `ce setup` (or: claude plugin marketplace add {}, then claude plugin install {})",
            "未检测到 Claude Code——插件未接线；装好 Claude Code 后运行 `ce setup`（或：claude plugin marketplace add {}，再 claude plugin install {}）",
            &[&source, &target],
        ),
        (false, Exit::AddFailed) => line(
            "marketplace registration failed — {}; run yourself: claude plugin marketplace add {}, then claude plugin install {}",
            "marketplace 注册失败——{}；请自行运行：claude plugin marketplace add {}，再 claude plugin install {}",
            &[&err, &source, &target],
        ),
        (false, Exit::InstallFailed) => line(
            "plugin install failed — {}; run yourself: claude plugin install {}",
            "插件安装失败——{}；请自行运行：claude plugin install {}",
            &[&err, &target],
        ),
        (false, Exit::Kept) => line(
            "Claude Code at {} — marketplace {} already registered (kept); {} installed and refreshed — restart Claude Code sessions to activate",
            "Claude Code 位于 {}——marketplace {} 已注册（保留）；{} 已安装并刷新——重启 Claude Code 会话即生效",
            &[&s("claude"), &market, &target],
        ),
        (false, Exit::Wired) => line(
            "Claude Code at {} — plugin wired: marketplace {} registered, {} installed — restart Claude Code sessions to activate",
            "Claude Code 位于 {}——插件已接线：marketplace {} 已注册，{} 已安装——重启 Claude Code 会话即生效",
            &[&s("claude"), &source, &target],
        ),
        (true, Exit::Wired) if d["marker"]["before"] == false => line(
            "not wired by `ce setup` — nothing removed (a registration you made stays yours)",
            "不是 `ce setup` 接的线——什么都没移除（你自己做的注册仍归你）",
            &[],
        ),
        (true, Exit::Wired) => line(
            "plugin unwired: {} uninstalled, marketplace {} removed",
            "插件已断开：{} 已卸载，marketplace {} 已移除",
            &[&target, &market],
        ),
        (true, _) => line(
            "unwire incomplete — {}; remove yourself: claude plugin uninstall {}, then claude plugin marketplace remove {}",
            "断开未完成——{}；请自行运行：claude plugin uninstall {}，再 claude plugin marketplace remove {}",
            &[&err, &target, &market],
        ),
    }
}
