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
    /// 1-based line number on the line's OWN side, tracked from the
    /// hunk headers (attack review F2: count-only ground truth could
    /// not see a line-identity substitution).
    pub line: usize,
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

/// `@@ -old[,n] +new[,k] @@` → the two raw range fields.
fn hunk_ranges(plain: &str) -> Option<(&str, &str)> {
    let rest = plain.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(" +")?;
    let (new, _) = rest.split_once(" @@")?;
    Some((old, new))
}

/// Hunk header → the two 1-based starts.
fn hunk_starts(plain: &str) -> Option<(usize, usize)> {
    let (old, new) = hunk_ranges(plain)?;
    let num = |s: &str| s.split(',').next()?.parse().ok();
    Some((num(old)?, num(new)?))
}

/// Hunk header → the two span lengths (omitted = 1).
fn hunk_spans(plain: &str) -> Option<(u64, u64)> {
    let (old, new) = hunk_ranges(plain)?;
    let span = |s: &str| match s.split_once(',') {
        Some((_, n)) => n.parse().ok(),
        None => Some(1),
    };
    Some((span(old)?, span(new)?))
}

/// Per-file-section (deleted, added) totals of a plain `-U0` diff,
/// summed from its hunk headers — the second, body-independent
/// reading the conservation assert compares against the colored
/// walk. Replaces numstat: git's default myers can overcount there
/// against its own patch (raw vs compacted edit script — requests
/// 28d537dd reads 15/6 by numstat, 14/5 by patch, difflib, minimal,
/// patience and histogram alike).
pub fn hunk_totals(raw: &str) -> Vec<(Option<String>, Option<String>, u64, u64)> {
    let mut out: Vec<(Option<String>, Option<String>, u64, u64)> = Vec::new();
    let mut a_path: Option<String> = None;
    for line in raw.lines() {
        if let Some(p) = line.strip_prefix("--- ") {
            a_path = header_path(p);
        } else if let Some(p) = line.strip_prefix("+++ ") {
            out.push((a_path.take(), header_path(p), 0, 0));
        } else if let Some((del, add)) = hunk_spans(line) {
            let s = out.last_mut().expect("hunk before file header");
            s.2 += del;
            s.3 += add;
        }
    }
    out
}

/// Walk a force-colored `git diff`: track file sections from their
/// `---`/`+++` headers, line counters from the hunk headers, and
/// classify every -/+ body line. Sections without those headers
/// (pure rename, mode change) have no body lines by construction.
pub fn walk_color_diff(raw: &str) -> Vec<BodyLine> {
    let mut out = Vec::new();
    let (mut a_path, mut b_path): (Option<String>, Option<String>) = (None, None);
    let (mut old_next, mut new_next) = (0usize, 0usize);
    for line in raw.lines() {
        let plain = strip_sgr(line);
        if plain.starts_with("diff --git ") {
            (a_path, b_path) = (None, None);
        } else if let Some(p) = plain.strip_prefix("--- ") {
            a_path = header_path(p);
        } else if let Some(p) = plain.strip_prefix("+++ ") {
            b_path = header_path(p);
        } else if let Some((old, new)) = hunk_starts(&plain) {
            (old_next, new_next) = (old, new);
        } else if let Some((added, moved)) = body_class(line) {
            let counter = if added { &mut new_next } else { &mut old_next };
            let line_no = *counter;
            *counter += 1;
            out.push(BodyLine {
                a_path: a_path.clone(),
                b_path: b_path.clone(),
                added,
                moved,
                content: plain[1..].to_string(),
                line: line_no,
            });
        }
    }
    out
}

/// The commit-slice scope: the five supported languages, minus
/// machine-local `memory/` state (also the M7 filter-repo surface).
/// This is a canonical-extension benchmark on purpose — variant
/// suffixes (.tsx/.mts/.markdown…) stay out on every corpus, and the
/// self-specific excludes are inert on foreign repos without such
/// paths; one frozen scope keeps corpora comparable.
pub const COMMIT_SCOPE: [&str; 7] = [
    "*.rs",
    "*.py",
    "*.ts",
    "*.go",
    "*.md",
    ":(exclude)memory/**",
    ":(exclude)cli/memory/**",
];

/// Run git against the corpus repository, optionally restricted to
/// the commit-slice scope. Asserts success.
///
/// The corpus is the enclosing repository unless `CE_SLICE_REPO`
/// names another one (M5-1 external validation; absolute path, same
/// convention as fpr_replay's CE_FPR_REPO). Every slice instrument
/// funnels its git here, so the one variable retargets slice
/// generation, prelabels, and replay together — while doc provenance
/// (generated_from) keeps its own git call bound to the enclosing
/// repo on purpose: it records the instrument, not the corpus.
pub fn git_run(args: &[&str], scoped: bool) -> String {
    // renameLimit=0 = unlimited (verified on git 2.52): at the default
    // limit a big commit silently degrades -M -C pairing into D+A,
    // warning only on stderr with exit 0 — which the success path
    // used to discard. Belt and braces: lift the limit AND refuse any
    // success-with-stderr (GT generation has no benign warnings).
    // The algorithm stays default myers: the frozen self slice and its
    // reviewed GT chain were generated under it (histogram regen
    // drifts the doc), so counts are derived from hunk headers of the
    // same script instead of numstat — see hunk_totals.
    let repo = std::env::var("CE_SLICE_REPO").ok();
    let mut full = vec!["-c", "diff.renameLimit=0"];
    full.extend_from_slice(args);
    if scoped {
        full.push("--");
        full.extend_from_slice(&COMMIT_SCOPE);
    }
    git_in(repo.as_deref(), &full)
}

/// Run git in `repo` (None = the enclosing repository) with the slice
/// instruments' success discipline: a non-zero exit OR any stderr on
/// success refuses (see git_run — a git warning on the success path
/// is a silently degraded result). The per-call repo argument exists
/// for the graph-sample pool walk, which reads several pinned corpora
/// inside one process and so cannot ride the CE_SLICE_REPO env var.
pub fn git_in(repo: Option<&str>, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    if let Some(repo) = repo {
        cmd.arg("-C").arg(repo);
    }
    let out = cmd.args(args).output().expect("git");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "git {args:?}: {stderr}");
    assert!(stderr.trim().is_empty(), "git {args:?} warned: {stderr}");
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
