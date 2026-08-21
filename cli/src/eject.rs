//! `ce eject` (plan §5.9-4, M7-P2): the uninstall path. Shuts the
//! project daemon down and — ONLY once it has let go (`released`) —
//! removes the analysis cache (.ce/), the committed baseline, and any
//! pinned starter binaries in CLAUDE_PLUGIN_DATA. Dry-run by default
//! — destructive removal only under --yes, and every target is named
//! either way: a delete that cannot be previewed is a delete nobody
//! trusts. ce.toml stays (it is the user's declaration, not our state
//! — §5.9-4 lists it nowhere).

use crate::daemon::{
    client,
    proto::{Request, Response},
};
use crate::i18n::line;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(root: &Path, yes: bool) -> ExitCode {
    let targets = targets(root);
    if targets.is_empty() {
        println!(
            "{}",
            crate::i18n::t(
                "eject: nothing to remove (no .ce/, baseline, or pinned binaries)",
                "eject：无可移除项（没有 .ce/、基线或钉扎二进制）",
            )
        );
        return ExitCode::SUCCESS;
    }
    if !yes {
        for t in &targets {
            println!(
                "{}",
                line("would remove: {}", "将移除：{}", &[&t.display()])
            );
        }
        println!(
            "{}",
            line(
                "eject: dry run — pass --yes to remove {} target(s)",
                "eject：试运行 — 传 --yes 以移除 {} 个目标",
                &[&targets.len()],
            )
        );
        return ExitCode::SUCCESS;
    }
    if !released(root) {
        eprintln!(
            "{}",
            line(
                "eject: a daemon is still serving {} — nothing removed; stop it and retry",
                "eject：仍有守护进程在服务 {} — 未移除任何目标；请先停止它再重试",
                &[&root.display()],
            )
        );
        return ExitCode::FAILURE;
    }
    remove_all(&targets)
}

/// Did the daemon actually let go? The destructive phase is gated on
/// this because the shutdown's outcome used to be discarded whole:
/// the request can lose the race against a freshly bound daemon's
/// token write (client.rs header, review 2026-08-20 #7), and eject
/// read that refusal as "nothing there" and deleted .ce out from
/// under a live daemon. Exactly two states are safe to delete in — a
/// Bye we were answered, or a socket nobody holds. Never the
/// lazy-spawn path: spawning a daemon in order to shut it down would
/// recreate the state being removed.
fn released(root: &Path) -> bool {
    let shutdown = client::request_if_running(root, &Request::Shutdown);
    matches!(shutdown, Ok(Response::Bye)) || !client::is_running(root)
}

/// The destructive phase (split at the 50-line fn gate): remove each
/// target, name every success and failure, and tally the exit.
fn remove_all(targets: &[PathBuf]) -> ExitCode {
    let mut failed = 0;
    for t in targets {
        match remove(t) {
            Ok(()) => println!("{}", line("removed: {}", "已移除：{}", &[&t.display()])),
            Err(e) => {
                failed += 1;
                eprintln!("eject: {} — {e}", t.display());
            }
        }
    }
    if failed > 0 {
        eprintln!(
            "{}",
            line(
                "eject: {} target(s) not removed",
                "eject：{} 个目标未移除",
                &[&failed],
            )
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Everything eject owns, existing entries only: the §5.9-4 list —
/// .ce/ (the project's AND any stray below it), ce-baseline.json,
/// pinned starter binaries (see `ours`).
fn targets(root: &Path) -> Vec<PathBuf> {
    let anchor = crate::root::project_root(root);
    let mut out = strays(&anchor);
    let ce = anchor.join(".ce");
    if ce.exists() {
        out.insert(0, ce);
    }
    let baseline = crate::score::baseline::path_for(root);
    if baseline.exists() {
        out.push(baseline);
    }
    if let Ok(data) = std::env::var("CLAUDE_PLUGIN_DATA")
        && let Ok(entries) = std::fs::read_dir(&data)
    {
        // is_file: a DIRECTORY is never a starter artifact, and --yes
        // deletes this list recursively
        for e in entries.flatten() {
            if ours(&e.file_name().to_string_lossy()) && e.file_type().is_ok_and(|t| t.is_file()) {
                out.push(e.path());
            }
        }
    }
    out
}

/// Nested `.ce/` directories below the project root. Before the
/// state throats learned to ascend, every run pointed at a
/// subdirectory minted one there — five existed in this repository,
/// invisible to `git status` behind an unanchored ignore rule, each
/// holding file paths, symbol keys and an observe feed. An uninstall
/// that leaves them behind is not the "full per-project uninstall"
/// the README promises. A directory carrying its own `ce.toml` is a
/// project in its own right and is left alone, along with the
/// build/vendor trees no walk should enter.
fn strays(anchor: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut queue = vec![anchor.to_path_buf()];
    while let Some(dir) = queue.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if !e.file_type().is_ok_and(|t| t.is_dir()) {
                continue;
            }
            let path = e.path();
            match e.file_name().to_string_lossy().as_ref() {
                // the project's own .ce is listed by the caller, in
                // first position — this walk reports only the strays
                ".ce" => {
                    if dir != anchor {
                        out.push(path);
                    }
                }
                ".git" | "target" | "node_modules" | "dist-newstyle" => {}
                _ if path.join("ce.toml").is_file() => {}
                _ => queue.push(path),
            }
        }
    }
    out.sort();
    out
}

/// Ours = `ce-core[.exe]` or `ce-<version>-…` (versions open with a
/// DIGIT); `starts_with("ce-")` also ate a neighbour's `ce-cache`.
fn ours(name: &str) -> bool {
    let Some(tail) = name.strip_prefix("ce-") else {
        return false;
    };
    tail == "core" || tail.starts_with("core.") || tail.starts_with(|c: char| c.is_ascii_digit())
}

/// Remove one target with a bounded teardown wait: the daemon just
/// received Shutdown and Windows keeps the index file locked until
/// its exiting process closes handles — this is ordering the
/// teardown (the M5 SQLite drop-before-delete lesson), not masking a
/// race: after 10 tries the error surfaces with the path named.
fn remove(path: &Path) -> std::io::Result<()> {
    let mut last = None;
    for _ in 0..10 {
        let r = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
    Err(last.expect("ten failed attempts recorded an error"))
}

#[cfg(test)]
mod tests {
    use super::{released, run};
    use interprocess::local_socket::traits::ListenerExt;
    use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
    use std::path::{Path, PathBuf};

    /// A daemon that holds the socket and answers nothing — what the
    /// token-write race looks like from the client: up, but not (yet)
    /// speaking for us. It accepts WITHOUT bound and is never joined:
    /// a listener that stopped early would unbind the socket, and a
    /// test whose stub disappears mid-run is a test that green-lights
    /// the very deletion it exists to forbid.
    fn deaf_daemon(root: &Path) {
        let ns = crate::daemon::proto::socket_name(root)
            .to_ns_name::<GenericNamespaced>()
            .expect("name");
        let listener = ListenerOptions::new().name(ns).create_sync().expect("bind");
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                drop(stream); // accepted, then hung up
            }
        });
    }

    /// A project with its own anchor, so `targets` resolves here and
    /// the stray walk stays inside the scratch dir.
    fn seeded(tag: &str) -> PathBuf {
        let root = crate::testutil::scratch(tag);
        std::fs::create_dir_all(root.join(".ce")).expect(".ce");
        for (name, body) in [("ce.toml", ""), ("ce-baseline.json", "{}")] {
            std::fs::write(root.join(name), body).expect(name);
        }
        root
    }

    /// --yes must remove NOTHING while a daemon still holds the
    /// socket: discarding the shutdown's outcome let eject delete .ce
    /// out from under a live daemon (review 2026-08-20 #7). State is
    /// the assertion — ExitCode has no PartialEq to compare.
    #[test]
    fn yes_keeps_every_target_while_a_daemon_holds_the_socket() {
        let root = seeded("eject-live-daemon");
        deaf_daemon(&root);
        // gate first: if it ever opens, the run below is the delete
        assert!(!released(&root), "an unanswered shutdown never releases");
        let _ = run(&root, true);
        assert!(root.join(".ce").exists(), "cache kept for the live daemon");
        assert!(root.join("ce-baseline.json").exists(), "baseline kept");
    }

    /// ...and the gate does not block the cold path it was added to
    /// protect: no socket, nobody to wait for. (Asserted through
    /// `released`, not `run`: a --yes run here would also sweep this
    /// process's real CLAUDE_PLUGIN_DATA.)
    #[test]
    fn a_root_with_no_daemon_is_released() {
        let root = crate::testutil::scratch("eject-no-daemon");
        assert!(released(&root), "an unheld socket releases at once");
    }
}
