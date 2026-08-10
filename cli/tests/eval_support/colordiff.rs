//! Force-colored git-diff parsing shared by the eval generators: SGR
//! line classification (`--color-moved` is the only interface git
//! exposes for line-level move detection) and the whole-commit walk.

use std::path::Path;
use std::process::Command;

/// Per-line four-class counts under `--color-moved` (SGR 31/32 =
/// deleted/novel, 35/36 = moved-out/moved-in).
#[derive(Default)]
pub struct LineClasses {
    pub added_novel: usize,
    pub added_moved: usize,
    pub removed_deleted: usize,
    pub removed_moved: usize,
}

impl LineClasses {
    pub fn count(&mut self, added: bool, moved: bool) {
        match (added, moved) {
            (true, true) => self.added_moved += 1,
            (true, false) => self.added_novel += 1,
            (false, true) => self.removed_moved += 1,
            (false, false) => self.removed_deleted += 1,
        }
    }
}

/// Split a line into its leading SGR parameter runs and the first
/// payload character. `\x1b[1;36m+` and `\x1b[36m+` must both read as
/// cyan (empirically git's moved defaults are the BOLD variants, and
/// the `---`/`+++` headers are bold with no color at all).
pub fn sgr_prefix(line: &str) -> (Vec<&str>, Option<char>) {
    let mut rest = line;
    let mut codes = Vec::new();
    while let Some(tail) = rest.strip_prefix("\x1b[") {
        let Some(end) = tail.find('m') else { break };
        codes.extend(tail[..end].split(';'));
        rest = &tail[end + 1..];
    }
    (codes, rest.chars().next())
}

/// One diff body line under `--color-moved` → (added, moved): 31/32 =
/// plain removed/added, 35/36 = moved out/in (bold or not); `None`
/// for anything else (hunk/file headers are never 31/32/35/36 + `-+`).
pub fn body_class(line: &str) -> Option<(bool, bool)> {
    let (codes, first) = sgr_prefix(line);
    let has = |code: &str| codes.contains(&code);
    match first {
        Some('-') if has("35") => Some((false, true)),
        Some('-') if has("31") => Some((false, false)),
        Some('+') if has("36") => Some((true, true)),
        Some('+') if has("32") => Some((true, false)),
        _ => None,
    }
}

/// Classify one diff body line into running four-class counts.
pub fn classify_line(line: &str, c: &mut LineClasses) {
    if let Some((added, moved)) = body_class(line) {
        c.count(added, moved);
    }
}

/// Remove every SGR escape (`\x1b[…m`) anywhere in the line — for
/// parsing file headers out of force-colored whole-commit diffs.
pub fn strip_sgr(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find("\x1b[") {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 2..];
        match tail.find('m') {
            Some(end) => rest = &tail[end + 1..],
            None => rest = "",
        }
    }
    out.push_str(rest);
    out
}

/// One attributed body line of a force-colored whole-commit diff.
pub struct BodyLine {
    pub a_path: Option<String>,
    pub b_path: Option<String>,
    pub added: bool,
    pub moved: bool,
    /// SGR-stripped content without the leading `-`/`+`.
    pub content: String,
}

impl BodyLine {
    /// The file this line sits in on its own side.
    pub fn own_path(&self) -> &str {
        let side = if self.added {
            &self.b_path
        } else {
            &self.a_path
        };
        side.as_deref().expect("body line owner")
    }
}

fn header_path(p: &str) -> Option<String> {
    (p != "/dev/null").then(|| {
        let p = p
            .strip_prefix("a/")
            .or_else(|| p.strip_prefix("b/"))
            .unwrap_or(p);
        p.to_string()
    })
}

/// Walk a force-colored `git diff`: track file sections from their
/// `---`/`+++` headers and classify every -/+ body line. Sections
/// without those headers (pure rename, mode change) have no body
/// lines by construction.
pub fn walk_color_diff(raw: &str) -> Vec<BodyLine> {
    let mut out = Vec::new();
    let (mut a_path, mut b_path): (Option<String>, Option<String>) = (None, None);
    for line in raw.lines() {
        let plain = strip_sgr(line);
        if plain.starts_with("diff --git ") {
            (a_path, b_path) = (None, None);
        } else if let Some(p) = plain.strip_prefix("--- ") {
            a_path = header_path(p);
        } else if let Some(p) = plain.strip_prefix("+++ ") {
            b_path = header_path(p);
        } else if let Some((added, moved)) = body_class(line) {
            out.push(BodyLine {
                a_path: a_path.clone(),
                b_path: b_path.clone(),
                added,
                moved,
                content: plain[1..].to_string(),
            });
        }
    }
    out
}

/// The commit-slice scope: the five supported languages, minus
/// machine-local `memory/` state (also the M7 filter-repo surface).
pub const COMMIT_SCOPE: [&str; 7] = [
    "*.rs",
    "*.py",
    "*.ts",
    "*.go",
    "*.md",
    ":(exclude)memory/**",
    ":(exclude)cli/memory/**",
];

/// Run git against the enclosing repository, optionally restricted to
/// the commit-slice scope. Asserts success.
pub fn git_run(args: &[&str], scoped: bool) -> String {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if scoped {
        cmd.arg("--");
        cmd.args(COMMIT_SCOPE);
    }
    let out = cmd.output().expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The commit slice's force-colored whole-commit diff (blocks mode).
pub fn commit_color_diff(base: &str, sha: &str) -> String {
    #[rustfmt::skip]
    let args = ["-c", "color.diff=always", "diff", "-U0", "-M", "-C",
        "--color-moved=blocks", "--color-moved-ws=allow-indentation-change",
        base, sha];
    git_run(&args, true)
}

/// `git diff --no-index --numstat [extra…] a b` → (added, deleted).
/// `extra` lets the baseline pass the plan-literal `-M -C
/// --find-copies-harder` while the prelabel pass runs plain.
pub fn numstat(a: &Path, b: &Path, extra: &[&str]) -> (u64, u64) {
    let out = Command::new("git")
        .args(["diff", "--no-index", "--numstat"])
        .args(extra)
        .arg(a)
        .arg(b)
        .output()
        .expect("git diff numstat");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut fields = text.split_whitespace();
    let add = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let del = fields.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (add, del)
}
