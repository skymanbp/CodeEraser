"use strict";
// The scratch-tree plumbing the demo family shares: how a copy of the
// seed is made, committed and given a baseline; how the guard is
// asked; how a scratch path is kept out of a transcript.
//
// run.js replays the whole task twice through it and vignettes.js
// builds one small scene per exhibit. Neither owns the other and
// neither carries a copy — which this repo's own dedup gate could not
// have enforced, since JavaScript is on the size-only arm here
// (Lang::scan_only): a second copy of `seedTree` would have been
// invisible to every gate and visible to every reader.

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const CE = process.env.CE_BIN || "ce";

function slashed(p) {
  return p.replace(/\\/g, "/");
}

/** Run one process to completion; never throws on a non-zero exit. */
function run(cmd, args, cwd, input, extra = {}) {
  const env = { ...process.env, CE_LANG: "en", CE_PROGRESS: "0", CE_DAEMON_IDLE_SECS: "60", ...extra };
  const r = cp.spawnSync(cmd, args, { cwd, input, encoding: "utf8", env, shell: process.platform === "win32" && cmd === CE && !path.isAbsolute(CE) });
  if (r.error) throw r.error;
  return { out: (r.stdout || "") + (r.stderr || ""), rc: r.status };
}

// a fixed authorship makes every commit object reproducible: `ce erase
// --apply` wants a clean worktree, so the demo commits, and a wall clock
// in the tree would be one more thing that could differ between runs
const WHEN = "2026-01-01T00:00:00+00:00";

function git(cwd, ...args) {
  const cfg = ["-c", "user.name=demo", "-c", "user.email=demo@example.com", "-c", "core.autocrlf=false"];
  return run("git", [...cfg, ...args], cwd, undefined, { GIT_AUTHOR_DATE: WHEN, GIT_COMMITTER_DATE: WHEN });
}

/** A fresh copy of the seed, committed, with its baseline established. */
function seedTree(dir, seed) {
  for (const [rel, text] of Object.entries(seed)) {
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.writeFileSync(path.join(dir, rel), text);
  }
  git(dir, "init", "-q");
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "seed");
  // the one act that creates a missing baseline file, by name
  const r = run(CE, ["baseline", "."], dir, undefined, { CE_ACCEPT_BASELINE: "1" });
  if (r.rc !== 0) throw new Error(`baseline: ${r.out}`);
}

/** The PreToolUse envelope Claude Code sends for a Write. */
function writeEnvelope(dir, rel, content) {
  return JSON.stringify({
    session_id: "demo",
    transcript_path: "demo",
    cwd: slashed(dir),
    hook_event_name: "PreToolUse",
    tool_name: "Write",
    tool_input: { file_path: slashed(path.join(dir, rel)), content },
    tool_use_id: "demo",
  });
}

/** Ask the guard; {decision, reason} or null when it stays silent (allow). */
function probe(dir, rel, content, lang = "en") {
  const r = run(CE, ["probe", "--hook"], dir, writeEnvelope(dir, rel, content), { CE_LANG: lang });
  const line = r.out.trim();
  if (!line) return null;
  const v = JSON.parse(line).hookSpecificOutput || {};
  return { decision: v.permissionDecision, reason: v.permissionDecisionReason || "" };
}

/** Scratch paths never reach the transcript. */
function normalize(text, dir) {
  return text.split(slashed(dir)).join("<work>").split(dir).join("<work>").replace(/\r\n/g, "\n").trimEnd();
}

/** eject shuts the daemon (Bye => it exits on its own clock), removes .ce/ */
function ejectTree(dir) {
  const e = run(CE, ["eject", ".", "--yes"], dir);
  if (e.rc !== 0) throw new Error(`eject: ${e.out}`);
}

module.exports = { CE, WHEN, slashed, run, git, seedTree, writeEnvelope, probe, normalize, ejectTree };
