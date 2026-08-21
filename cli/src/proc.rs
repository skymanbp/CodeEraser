//! The ONE Command constructor for every child the product spawns
//! (ce-core, git, the daemon). The GUI links this crate directly and
//! its release build is a WINDOWED-subsystem process: on Windows each
//! console-subsystem child then gets a fresh console window unless
//! CREATE_NO_WINDOW is set — one trend run walks dozens of commits
//! and flashed a console per git call and core spawn on the user's
//! desktop (v0.7.3 root fix). Every shipped site pipes or nulls the
//! child's stdio, so no child ever needed the console this flag
//! suppresses; console parents (the CLI, hooks) are unaffected —
//! their children attach to the existing console and never created a
//! window to begin with.

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// `Command::new` with the no-console flag on Windows. Shipped spawn
/// sites go through here instead of `Command::new` — the four call
/// sites grew the same omission independently, so the flag lives in
/// exactly one place.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// The ONE git invocation (v0.7.3: the two per-family runners grew
/// into a 52-token clone pair the moment both routed through
/// `command`, and the repo's own dedup gate refused it — so the
/// shared body lives here and each family keeps only its error
/// stance). core.quotePath (git's DEFAULT) C-quotes non-ASCII paths
/// and every consumer joins git's answer against ce's own rel_str
/// spelling — a quoted path matches nothing and fails SILENTLY (the
/// Stop deny gate skipped CJK filenames); harmless for -z consumers,
/// whose output is already literal. stdin is the null device: no
/// caller feeds git input, and `hash-object --stdin` wants EOF.
pub fn git_output(root: &Path, args: &[&str]) -> std::io::Result<Output> {
    command("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .stdin(Stdio::null())
        .output()
}

#[cfg(test)]
mod tests {
    /// The flag must not cost the pipes: a child built here still
    /// answers over captured stdio — the transport every consumer
    /// (corelink NDJSON, the git runners) rides on. git is already a
    /// hard dependency of the product and of the churn battery.
    #[test]
    fn command_still_pipes_child_output() {
        let out = super::command("git")
            .arg("--version")
            .output()
            .expect("spawn git through the throat");
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("git version"));
    }
}
