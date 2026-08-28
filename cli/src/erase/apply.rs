//! The destructive phase's executor (erase.md §two-phases):
//! preconditions in contract order — a git repository (revert is one
//! checkout away), a CLEAN worktree (never entangled with
//! uncommitted work), every target's bytes still the plan's — then
//! the writes and the append-only audit record. The convergence
//! re-plan lives in mod.rs (apply_plan), which calls DOWN into this
//! executor — never the other way. Only files the plan named are
//! ever touched; the log is part of the contract, so unlike the
//! observe feed its failures are errors, never swallowed telemetry.

use crate::erase::model::{Plan, Row};
use anyhow::{Context, Result, ensure};
use std::path::Path;

/// Bump-log: 0.1.0 = the M9 batch-3 landing shape.
const LOG_SCHEMA: &str = "ce.erase-log/0.1.0";

/// Preconditions + writes + audit log. Returns the applied count.
pub(super) fn execute(root: &Path, plan: &Plan) -> Result<usize> {
    let targets: Vec<&Row> = plan.eraseable().collect();
    if targets.is_empty() {
        return Ok(0);
    }
    preconditions(root, &targets)?;
    let plan_hash = crate::dedup::tokens::fnv1a(
        serde_json::to_string(&plan.rows)
            .expect("plan rows")
            .as_bytes(),
    );
    for chunk in per_file(&targets) {
        write_target(root, &chunk)?;
        for r in chunk {
            append_log(root, r, plan_hash)?;
        }
    }
    Ok(targets.len())
}

/// Contract order, each refusal by name, nothing written before all
/// three hold. The repository check demands the resolved toplevel
/// EQUAL the erase root: git ascends past an anchorless dir into any
/// enclosing repository (the hook root-anchoring class), and an
/// apply silently bound to a parent repo would pass ITS cleanliness
/// while erasing under a root git never vouched for.
fn preconditions(root: &Path, targets: &[&Row]) -> Result<()> {
    let git = |args: &[&str]| crate::proc::git_output(root, args);
    let top = git(&["rev-parse", "--show-toplevel"]).context("git")?;
    ensure!(
        top.status.success(),
        "not a git repository — an erase must be one `git checkout` from revert"
    );
    let top = std::path::PathBuf::from(String::from_utf8_lossy(&top.stdout).trim());
    let same = match (top.canonicalize(), root.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    ensure!(
        same,
        "not a git repository at {} — git resolved an enclosing repository \
         at {} instead, and an erase never binds past its own root",
        root.display(),
        top.display()
    );
    let dirty = git(&["status", "--porcelain"]).context("git status")?;
    ensure!(dirty.status.success(), "git status failed");
    // ce's own .ce/ state dir is not "uncommitted work" (the audit's
    // ce_owned exclusion, changes.rs): planning itself refreshes the
    // index, and a precondition the plan trips by existing would make
    // apply unreachable in any repo that does not gitignore .ce/.
    let user_dirt = String::from_utf8_lossy(&dirty.stdout)
        .lines()
        .map(str::to_string)
        .filter(|l| {
            let p = l.get(3..).unwrap_or("").trim_start_matches('"');
            !(p == ".ce" || p == ".ce/" || p.starts_with(".ce/"))
        })
        .collect::<Vec<_>>();
    ensure!(
        user_dirt.is_empty(),
        "worktree not clean — commit or stash first; an erase must never \
         be entangled with uncommitted work ({})",
        user_dirt.join(", ")
    );
    let mut cache = crate::erase::gather::TextCache::new(root);
    for r in targets {
        let now = crate::dedup::tokens::fnv1a(cache.text(&r.path)?.as_bytes());
        ensure!(
            now == r.hash,
            "{}: content changed since planning — plans are not portable \
             across edits; re-run ce erase",
            r.path
        );
    }
    Ok(())
}

/// Plan-ordered rows grouped per target file.
fn per_file<'a>(targets: &[&'a Row]) -> Vec<Vec<&'a Row>> {
    let mut out: Vec<Vec<&'a Row>> = Vec::new();
    for r in targets {
        match out.last_mut() {
            Some(chunk) if chunk[0].path == r.path => chunk.push(r),
            _ => out.push(vec![r]),
        }
    }
    out
}

/// One file's writes: a whole-file row deletes it; span rows splice
/// in DESCENDING span order so earlier line numbers stay valid.
fn write_target(root: &Path, chunk: &[&Row]) -> Result<()> {
    let file = root.join(&chunk[0].path);
    if chunk.iter().any(|r| r.span.is_none()) {
        return std::fs::remove_file(&file).with_context(|| chunk[0].path.clone());
    }
    let text = std::fs::read_to_string(&file).with_context(|| chunk[0].path.clone())?;
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut spans: Vec<(i64, i64)> = chunk.iter().filter_map(|r| r.span).collect();
    spans.sort_unstable_by(|a, b| b.cmp(a));
    for (s, e) in spans {
        let (a, b) = ((s.max(1) - 1) as usize, (e as usize).min(lines.len()));
        ensure!(a < b, "{}: empty span {s}-{e}", chunk[0].path);
        lines.drain(a..b);
    }
    std::fs::write(&file, lines.concat()).with_context(|| chunk[0].path.clone())
}

/// One audit record, one write_all (the observe-feed interleaving
/// lesson, hookio.rs) — but errors PROPAGATE: this log is the apply
/// contract's audit trail, not telemetry.
fn append_log(root: &Path, r: &Row, plan_hash: u64) -> Result<()> {
    use std::io::Write;
    let dir = root.join(".ce");
    std::fs::create_dir_all(&dir)?;
    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let line = serde_json::json!({
        "schema": LOG_SCHEMA,
        "ts_ms": ts_ms,
        "class": r.class,
        "path": r.path,
        "span": r.span,
        "provenance": r.provenance,
        "hash": format!("{:016x}", r.hash),
        "plan": format!("{plan_hash:016x}"),
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("erase-log.ndjson"))?;
    f.write_all(format!("{line}\n").as_bytes())?;
    Ok(())
}
