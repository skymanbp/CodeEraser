#!/usr/bin/env node
// The pin step of the two-phase release (docs/RELEASE.md §2.1), as a
// program instead of a transcription: read the draft's SHA256SUMS, put
// the measured hashes — one ce, one ce-core, one GUI bundle per roster
// target (scripts/roster.js) — into plugin/bin/manifest.env beside the
// new CE_MANIFEST_VERSION and CE_BASE_URL, regenerate the package-
// manager projections under packaging/ (scripts/packaging.js), and
// verify that the manifest now pins exactly this draft.
// Hand-copying these lines shipped a wrong pin once and a stale base
// URL once; each release the same scratch script was rewritten from
// memory (revival O77).
//
// The verdict reads the FINAL STATE, never a diff against HEAD. The
// first version asserted a moved-line count from `git diff` (HEAD vs
// the working file) while deciding how many lines to expect from the
// WORKING file alone: run it twice and the second run saw a manifest
// already at the new version, expected fifteen lines and was shown the
// seventeen still standing against HEAD — an identical retry, and a
// `--bless` re-run over an uncommitted bump, refused a correct tree.
// Every count printed below now comes from one consistent pair, the
// bytes read and the bytes written, both in this process.
//
// Usage: node scripts/pin_release.js <version> [--bless]
//   <version>  bare, e.g. 1.7.0 — the draft release is v<version>
//   --bless    also refresh the docs-facts line: contracts/docs-facts.json
//              carries `ver:pin#v`, derived from CE_MANIFEST_VERSION
//              (the BLESS command below, run here)
// Exit 0 = the manifest now pins the draft; anything else refused by name.
"use strict";
const { execFileSync, spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { TARGETS, WHAT, asset, pinKey, parseManifest } = require("./roster");

const ROOT = path.join(__dirname, "..");
/** An offline stand-in for the draft, for the exercises in
 *  tests/it/pin_release.rs: `<dir>/draft/` holds the asset names and the
 *  SHA256SUMS text, `<dir>/manifest.env` is the disposable copy this run
 *  rewrites, and the sibling generators are NAMED instead of run (they
 *  project the real manifest, which an exercise must not touch). Unset
 *  on every release path — `gh` and plugin/bin/manifest.env then. */
const SANDBOX = process.env.CE_PIN_SANDBOX || "";
const MANIFEST = SANDBOX
  ? path.join(SANDBOX, "manifest.env")
  : path.join(ROOT, "plugin", "bin", "manifest.env");
/** Pins per release: three artifacts per target; assets: those plus SHA256SUMS. */
const PINS = TARGETS.length * WHAT.length;
const ASSETS = PINS + 1;
/** The docs-facts bless, spelled ONCE: the argv this script runs and the
 *  line a human pastes at the repository root are the same command. It
 *  was not — the printed hint said `cargo test --test it`, which at the
 *  root finds no Cargo.toml at all (docs/RELEASE.md §2.1 carries the
 *  same spelling). Cargo runs the test binary with its package as cwd
 *  whatever directory cargo itself was invoked from, so ROOT plus
 *  --manifest-path is the same run as cli/ plus nothing. */
const BLESS = ["cargo", "test", "--manifest-path", "cli/Cargo.toml", "--test", "it", "--", "facts_"];
const BLESS_LINE = `CE_BLESS=1 ${BLESS.join(" ")}`;

/** Asset name -> manifest key, for one version — the same names
 *  release.yml's verify-publish downloads by roster. */
function roster(ver) {
  const out = {};
  for (const key of TARGETS) for (const what of WHAT) out[asset(what, ver, key)] = pinKey(what, key);
  return out;
}

function baseUrl(tag) {
  return `https://github.com/skymanbp/CodeEraser/releases/download/${tag}`;
}

function gh(args) {
  return execFileSync("gh", args, { cwd: ROOT, encoding: "utf8" });
}

function fail(msg) {
  console.error(`pin_release: ${msg}`);
  process.exit(1);
}

/** The draft's asset names: from the release, or from the stand-in. */
function draftNames(tag) {
  if (SANDBOX) return fs.readdirSync(path.join(SANDBOX, "draft")).sort();
  const view = JSON.parse(gh(["release", "view", tag, "--json", "isDraft,assets"]));
  if (view.isDraft !== true) fail(`${tag} is not a draft — a published release is never re-pinned`);
  return view.assets.map((a) => a.name).sort();
}

/** The draft must carry exactly the roster's assets. */
function draftAssets(tag) {
  const names = draftNames(tag);
  if (names.length !== ASSETS) fail(`${tag} carries ${names.length} assets, expected ${ASSETS}: ${names.join(", ")}`);
  return names;
}

/** The draft's SHA256SUMS text. */
function sumsText(tag) {
  if (SANDBOX) return fs.readFileSync(path.join(SANDBOX, "draft", "SHA256SUMS"), "utf8");
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ce-pin-"));
  gh(["release", "download", tag, "--pattern", "SHA256SUMS", "--dir", dir, "--clobber"]);
  const text = fs.readFileSync(path.join(dir, "SHA256SUMS"), "utf8");
  fs.rmSync(dir, { recursive: true, force: true });
  return text;
}

/** SHA256SUMS parsed: name -> hash, refusing any name off the roster. */
function sums(tag, ver) {
  const want = roster(ver);
  const got = {};
  for (const line of sumsText(tag).split("\n")) {
    if (!line.trim()) continue;
    const m = /^([0-9a-f]{64}) [ *](\S+)$/.exec(line);
    if (!m) fail(`SHA256SUMS line is not '<sha256>  <name>': ${line}`);
    if (!want[m[2]]) fail(`SHA256SUMS names an asset off the roster: ${m[2]}`);
    got[m[2]] = m[1];
  }
  const missing = Object.keys(want).filter((n) => !got[n]);
  if (missing.length) fail(`SHA256SUMS lacks: ${missing.join(", ")}`);
  return got;
}

/** Replace the one line `KEY="..."` in the manifest text; exactly one must exist. */
function setLine(text, key, value) {
  const re = new RegExp(`^${key}="[^"]*"$`, "m");
  const hits = text.match(new RegExp(re.source, "gm")) || [];
  if (hits.length !== 1) fail(`manifest has ${hits.length} lines for ${key}, expected 1`);
  return text.replace(re, `${key}="${value}"`);
}

/** Every complaint about the written manifest, spelled out. A partial
 *  manifest — one pin empty, one key gone, one hash off the draft —
 *  lands here as a named line; the tag phase requires complete pins, so
 *  the pin step refuses to leave anything less behind. */
function complaints(ver, tag, hashes) {
  const m = parseManifest(fs.readFileSync(MANIFEST, "utf8"));
  const want = roster(ver);
  const bad = [];
  if (m.CE_MANIFEST_VERSION !== ver) bad.push(`CE_MANIFEST_VERSION="${m.CE_MANIFEST_VERSION}", expected ${ver}`);
  if (m.CE_BASE_URL !== baseUrl(tag)) bad.push(`CE_BASE_URL="${m.CE_BASE_URL}", expected ${baseUrl(tag)}`);
  const got = Object.keys(m).filter((k) => k.startsWith("CE_SHA256_")).sort();
  const keys = Object.values(want).sort();
  if (got.join(" ") !== keys.join(" ")) bad.push(`pin keys are [${got.join(" ")}], the roster's are [${keys.join(" ")}]`);
  for (const [name, key] of Object.entries(want)) {
    const pin = m[key] || "";
    if (!/^[0-9a-f]{64}$/.test(pin)) bad.push(`${key}="${pin}" is not a 64-hex pin`);
    else if (pin !== hashes[name]) bad.push(`${key} pins ${pin}, the draft's ${name} is ${hashes[name]}`);
  }
  return bad;
}

/** Lines that differ between the bytes read and the bytes written — one
 *  consistent pair, so the number describes THIS run and nothing else. */
function moved(before, after) {
  const a = before.split("\n");
  const b = after.split("\n");
  let n = 0;
  for (let i = 0; i < Math.max(a.length, b.length); i += 1) if (a[i] !== b[i]) n += 1;
  return n;
}

/** A sibling script, inheriting stdio; a non-zero exit refuses by name. */
function sibling(script, args, env) {
  if (SANDBOX) return console.log(`sandbox: would run scripts/${script} ${args.join(" ")}`);
  const r = spawnSync(process.execPath, [path.join(__dirname, script), ...args], {
    cwd: ROOT,
    stdio: "inherit",
    env: { ...process.env, ...env },
  });
  if (r.status !== 0) fail(`${script} exited ${r.status}`);
}

/** The twelfth line: contracts/docs-facts.json's `ver:pin#v` is derived
 *  from CE_MANIFEST_VERSION, so the pin commit carries it or the pin
 *  commit is red (docs/RELEASE.md §2.1, twice learned). */
function bless() {
  console.log("refreshing the docs-facts line (contracts/docs-facts.json ver:pin#v) …");
  if (SANDBOX) return console.log(`sandbox: would run ${BLESS_LINE}`);
  const r = spawnSync(BLESS[0], BLESS.slice(1), {
    cwd: ROOT,
    stdio: "inherit",
    env: { ...process.env, CE_BLESS: "1" },
    shell: process.platform === "win32",
  });
  if (r.status !== 0) fail("the facts bless did not pass");
}

function main() {
  const ver = process.argv[2];
  if (!/^\d+\.\d+\.\d+$/.test(ver || "")) fail("usage: node scripts/pin_release.js <N.N.N> [--bless]");
  const tag = `v${ver}`;
  draftAssets(tag);
  const hashes = sums(tag, ver);

  const before = fs.readFileSync(MANIFEST, "utf8");
  let text = before;
  text = setLine(text, "CE_MANIFEST_VERSION", ver);
  text = setLine(text, "CE_BASE_URL", baseUrl(tag));
  for (const [name, key] of Object.entries(roster(ver))) text = setLine(text, key, hashes[name]);
  if (text !== before) fs.writeFileSync(MANIFEST, text);

  // the proof is the manifest itself, not the intent: every roster pin
  // present, 64-hex, equal to the draft's own hash, both version lines
  // naming this tag — and nothing off the roster
  const bad = complaints(ver, tag, hashes);
  if (bad.length) fail(`${MANIFEST} does not pin ${tag}:\n  ${bad.join("\n  ")}`);
  const n = moved(before, text);
  console.log(n ? `pinned ${tag}: ${n} line(s) moved` : `already pinned to ${tag}: no line moved`);
  // the Homebrew formula and the winget manifests are projections of
  // the manifest just written — they move with it, in the same commit
  sibling("packaging.js", [], {});

  if (process.argv.includes("--bless")) bless();
  else console.log(`docs-facts line: ${BLESS_LINE}   (or re-run with --bless)`);
}

main();
