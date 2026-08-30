#!/usr/bin/env node
"use strict";
// The same task, run twice — once with no CodeEraser in the loop, once
// with its PreToolUse guard and Stop audit in the loop — against
// identical copies of demo/seed, which is measured first so the zeros
// both runs start from are on the record. Each loop then runs to ITS
// end: without the hooks nothing refuses anything, so it ends at the
// last write; with them the audit refuses to end the turn, the repair
// it names is written, and `ce erase --apply` removes what the plan
// proves safe. Both trees are then measured by the same six commands.
// Nothing here is transcribed by hand: every verdict is the verbatim
// output of a `ce` subprocess, and the agent's moves are the scripted
// sequence in steps.js (no LLM is in the loop, which is what makes the
// two runs identical in everything except the hooks).
//
//   node demo/run.js            # run both, write demo/out/*
//   node demo/run.js --check    # run both, fail if demo/out/* or an embedded README table would change
//   node demo/bless.js          # after run.js: splice demo/out/summary*.md into the three README blocks
//
// Needs `ce` (CE_BIN or PATH) with a reachable ce-core (CE_CORE_BIN or a
// sibling), git, and node. No packages.

const fs = require("fs");
const os = require("os");
const path = require("path");
const cp = require("child_process");
const { steps, repair } = require("./steps");
const { renderSvg } = require("./render");
const { summaryTable } = require("./table");

const HERE = __dirname;
const SEED = path.join(HERE, "seed");
const OUT = path.join(HERE, "out");
const CE = process.env.CE_BIN || "ce";
const GATES = [
  ["check", "."],
  ["dedup", ".", "--check"],
  ["clone", "."],
  ["docdup", ".", "--check"],
  ["deadcode", ".", "--check"],
  ["erase", ".", "--check"],
];
const SHOWN_LINES = 8;

/** Every seed file as {relative path (slash-separated): text}. */
function readSeed(dir = SEED, prefix = "") {
  const files = {};
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const rel = prefix + entry.name;
    // state a tool left behind is not part of the seed: the plugin's
    // guard judges an edit under demo/seed at the seed's own root (it
    // carries a ce.toml) and leaves a .ce/ there
    if ([".ce", ".git", "__pycache__", "ce-baseline.json"].includes(entry.name)) continue;
    if (entry.isDirectory()) Object.assign(files, readSeed(path.join(dir, entry.name), rel + "/"));
    else files[rel] = fs.readFileSync(path.join(dir, entry.name), "utf8");
  }
  return files;
}

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
function probe(dir, rel, content) {
  const r = run(CE, ["probe", "--hook"], dir, writeEnvelope(dir, rel, content));
  const line = r.out.trim();
  if (!line) return null;
  const v = JSON.parse(line).hookSpecificOutput || {};
  return { decision: v.permissionDecision, reason: v.permissionDecisionReason || "" };
}

/** The Stop audit's block reason in one language, or null when it lets the turn end. */
function stopAudit(dir, lang) {
  const envelope = JSON.stringify({ session_id: "demo", transcript_path: "demo", cwd: slashed(dir), hook_event_name: "Stop", stop_hook_active: false });
  const r = run(CE, ["audit", "--hook"], dir, envelope, { CE_LANG: lang });
  const line = r.out.trim();
  return line ? JSON.parse(line).reason : null;
}

/** The audit is read-only, so it is asked once per language: each README's table
 *  then quotes the verdict in its own language instead of the English one. */
function stopAudits(dir) {
  return { en: stopAudit(dir, "en"), zh: stopAudit(dir, "zh") };
}

/** Scratch paths never reach the transcript. */
function normalize(text, dir) {
  return text.split(slashed(dir)).join("<work>").split(dir).join("<work>").replace(/\r\n/g, "\n").trimEnd();
}

/** One agent move: narrate, ask the guard when it is in the loop, write. */
function move(t, dir, step, withCe) {
  t.push({ kind: "agent", text: `${step.say}` });
  t.push({ kind: "cmd", text: `Write ${step.file}` });
  const verdict = withCe ? probe(dir, step.file, step.content) : null;
  if (verdict && verdict.decision === "deny") {
    t.push({ kind: "deny", text: `PreToolUse deny — ${normalize(verdict.reason, dir)}` });
    return false;
  }
  fs.mkdirSync(path.dirname(path.join(dir, step.file)), { recursive: true });
  fs.writeFileSync(path.join(dir, step.file), step.content);
  t.push({ kind: "allow", text: withCe ? "landed (the guard had nothing to say)" : "landed" });
  return true;
}

/** The six gates over a tree, each output clipped to its last lines. */
function measure(t, dir, summary) {
  for (const args of GATES) {
    const r = run(CE, [...args], dir);
    t.push({ kind: "cmd", text: `ce ${args.join(" ")}` });
    const out = normalize(r.out, dir);
    const lines = out.split("\n").filter((l) => !l.startsWith("advisory") && !/^[-+ ]/.test(l));
    for (const line of lines.slice(-SHOWN_LINES)) t.push({ kind: r.rc === 0 ? "out" : "red", text: line });
    summary.gates[args[0]] = { rc: r.rc, out };
  }
}

/** eject shuts the daemon (Bye => it exits on its own clock), removes .ce/ */
function ejectTree(dir) {
  const e = run(CE, ["eject", ".", "--yes"], dir);
  if (e.rc !== 0) throw new Error(`eject: ${e.out}`);
}

/** The seed under the same six gates — the zeros both runs start from, so
 *  the table can say whether a finding was already there or was written. */
function measureSeed(seed, work) {
  const dir = path.join(work, "seed");
  fs.mkdirSync(dir, { recursive: true });
  seedTree(dir, seed);
  const summary = { gates: {} };
  measure([], dir, summary);
  ejectTree(dir);
  return summary;
}

/** The rest of the loop, once the audit has refused to end the turn: the
 *  repair it named goes through the guard like any other write (removing
 *  duplication is never refused, and the run asserts it), the audit is
 *  asked again, and the tree is committed so `ce erase --apply` can act —
 *  its preconditions are a git repository, a clean worktree, unchanged
 *  targets. Nothing in the other run asks for any of this. */
function converge(t, dir, seed, summary) {
  if (!move(t, dir, repair(seed), true)) throw new Error("the repair was refused");
  t.push({ kind: "cmd", text: "Stop hook → ce audit --hook" });
  summary.stopAfterRepair = stopAudits(dir);
  const still = summary.stopAfterRepair.en;
  t.push(still ? { kind: "block", text: `Stop block — ${normalize(still, dir)}` } : { kind: "allow", text: "the turn may end" });
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "the task, as the audit let it end");
  const r = run(CE, ["erase", ".", "--apply"], dir);
  if (r.rc !== 0) throw new Error(`erase --apply: ${r.out}`);
  summary.gates.applied = { rc: r.rc, out: normalize(r.out, dir) };
  t.push({ kind: "cmd", text: "ce erase . --apply" });
  t.push({ kind: "out", text: summary.gates.applied.out.split("\n").pop() });
}

/** One complete run; returns {transcript, summary}. */
function runOnce(seed, withCe, work) {
  const dir = path.join(work, withCe ? "with" : "without");
  fs.mkdirSync(dir, { recursive: true });
  seedTree(dir, seed);
  const t = [{ kind: "note", text: withCe ? "CodeEraser in the loop: PreToolUse guard + Stop audit, ce.toml [guard] mode = \"deny\"" : "No CodeEraser in the loop: every write lands" }];
  const summary = { landed: 0, denied: 0, stop: null, stopAfterRepair: null, gates: {} };
  for (const step of steps(seed)) {
    if (move(t, dir, step, withCe)) summary.landed += 1;
    else summary.denied += 1;
  }
  t.push({ kind: "note", text: `session over: ${summary.landed} of ${summary.landed + summary.denied} writes landed` });
  if (!withCe) t.push({ kind: "note", text: "nothing refuses anything: the turn ends here" });
  else {
    t.push({ kind: "cmd", text: "Stop hook → ce audit --hook" });
    summary.stop = stopAudits(dir);
    t.push(summary.stop.en ? { kind: "block", text: `Stop block — ${normalize(summary.stop.en, dir)}` } : { kind: "allow", text: "the turn may end" });
    if (summary.stop.en) converge(t, dir, seed, summary);
  }
  t.push({ kind: "note", text: "the tree, measured (the CI face):" });
  measure(t, dir, summary);
  ejectTree(dir);
  return { transcript: t, summary };
}

function transcriptText(lines) {
  return lines.map((l) => ({ cmd: "$ ", agent: "agent> ", deny: "✗ ", block: "✗ ", allow: "✓ " }[l.kind] || "") + l.text).join("\n") + "\n";
}

/** The READMEs that embed a summary table between demo markers, and which one. */
const EMBEDS = [
  ["../README.md", "summary.md"],
  ["../README.zh.md", "summary.zh.md"],
  ["README.md", "summary.md"],
];

/** A README's marked block must be exactly the table this run produced. */
function embedDrift(files) {
  let drift = 0;
  for (const [rel, table] of EMBEDS) {
    const text = fs.readFileSync(path.join(HERE, rel), "utf8");
    const m = text.match(/<!-- demo:begin -->\n([\s\S]*?)<!-- demo:end -->/);
    if (!m || m[1] !== files[table]) { drift += 1; console.error(`demo: ${rel} demo block is stale`); }
  }
  return drift;
}

function main() {
  const check = process.argv.includes("--check");
  const seed = readSeed();
  const work = fs.mkdtempSync(path.join(os.tmpdir(), "codeeraser-demo-"));
  const start = measureSeed(seed, work);
  const without = runOnce(seed, false, work);
  const withCe = runOnce(seed, true, work);
  if (process.argv.includes("--keep")) console.log(`demo: all three trees kept under ${slashed(work)}`);
  else fs.rmSync(work, { recursive: true, force: true });
  const files = {
    "without-codeeraser.txt": transcriptText(without.transcript),
    "with-codeeraser.txt": transcriptText(withCe.transcript),
    "without-codeeraser.svg": renderSvg("the same task — without CodeEraser", without.transcript),
    "with-codeeraser.svg": renderSvg("the same task — with CodeEraser", withCe.transcript),
    "summary.md": summaryTable(start, without.summary, withCe.summary, "en"),
    "summary.zh.md": summaryTable(start, without.summary, withCe.summary, "zh"),
    "summary.json": JSON.stringify({ seed: start, without: without.summary, with: withCe.summary }, null, 2) + "\n",
  };
  fs.mkdirSync(OUT, { recursive: true });
  let drift = check ? embedDrift(files) : 0;
  for (const [name, text] of Object.entries(files)) {
    const target = path.join(OUT, name);
    if (check) {
      const have = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : null;
      if (have !== text) { drift += 1; console.error(`demo: ${name} would change`); }
    } else fs.writeFileSync(target, text);
  }
  console.log(check ? (drift ? `demo: ${drift} committed output(s) stale` : "demo: outputs match") : `demo: wrote ${Object.keys(files).length} files to ${slashed(OUT)}`);
  process.exit(drift ? 1 : 0);
}

if (require.main === module) main();
module.exports = { EMBEDS };
