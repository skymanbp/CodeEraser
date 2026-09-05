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
use std::process::{Child, Command, Output, Stdio};

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
/// caller here feeds git input, and `hash-object --stdin` wants EOF.
pub fn git_output(root: &Path, args: &[&str]) -> std::io::Result<Output> {
    git_command(root, args).stdin(Stdio::null()).output()
}

/// The one git invocation that FEEDS stdin: `cat-file --batch` reads
/// object names from it, and the tombstone legs ask for every changed
/// file's HEAD or index blob in ONE process rather than a `show` per
/// file (tombstone::texts). The input is written from its own thread:
/// git answers as it reads, so a writer that waited on the reply
/// could deadlock against a full stdout pipe. The reply comes back as
/// the RUNNING child, its stdout piped, for the caller to read as a
/// stream and then wait: collecting it whole (`wait_with_output`)
/// held every blob in memory before any size cap could look at it.
/// stderr is the null device — the exit status is the only error the
/// callers read.
pub fn git_feed(root: &Path, args: &[&str], input: &[u8]) -> std::io::Result<Child> {
    use std::io::Write as _;
    let mut child = git_command(root, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut stdin = child.stdin.take().expect("piped stdin");
    let input = input.to_vec();
    // detached: it ends when the input is written or the pipe closes
    std::thread::spawn(move || stdin.write_all(&input));
    Ok(child)
}

fn git_command(root: &Path, args: &[&str]) -> Command {
    let mut cmd = command("git");
    cmd.arg("-C")
        .arg(root)
        .args(["-c", "core.quotePath=false"])
        .args(args);
    cmd
}

#[cfg(test)]
#[path = "../tests/unit/proc.rs"]
mod tests;
