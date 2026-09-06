#!/usr/bin/env node
// The pin step of the two-phase release (docs/RELEASE.md §2.1), as a
// program instead of a transcription: read the draft's SHA256SUMS, put
// the measured hashes — one ce, one ce-core, one GUI bundle per roster
// target (scripts/roster.js) — into plugin/bin/manifest.env beside the
// new CE_MANIFEST_VERSION and CE_BASE_URL, regenerate the package-
// manager projections under packaging/ (scripts/packaging.js), and
// assert that exactly the manifest lines that must move did move.
// Hand-copying these lines shipped a wrong pin once and a stale base
// URL once; each release the same scratch script was rewritten from
// memory (revival O77).
//
// Usage: node scripts/pin_release.js <version> [--bless]
//   <version>  bare, e.g. 1.7.0 — the draft release is v<version>
//   --bless    also refresh the docs-facts line: contracts/docs-facts.json
//              carries `ver:pin#v`, derived from CE_MANIFEST_VERSION
//              (`CE_BLESS=1 cargo test --test it -- facts_`, run here)
// Exit 0 = the manifest now pins the draft; anything else refused by name.
"use strict";
const { execFileSync, spawnSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { TARGETS, WHAT, asset, pinKey } = require("./roster");

const ROOT = path.join(__dirname, "..");
const MANIFEST = path.join(ROOT, "plugin", "bin", "manifest.env");
/** Pins per release: three artifacts per target; assets: those plus SHA256SUMS. */
const PINS = TARGETS.length * WHAT.length;
const ASSETS = PINS + 1;

/** Asset name -> manifest key, for one version — the same names
 *  release.yml's verify-publish downloads by roster. */
function roster(ver) {
  const out = {};
  for (const key of TARGETS) for (const what of WHAT) out[asset(what, ver, key)] = pinKey(what, key);
  return out;
}

function gh(args) {
  return execFileSync("gh", args, { cwd: ROOT, encoding: "utf8" });
}

function fail(msg) {
  console.error(`pin_release: ${msg}`);
  process.exit(1);
}

/** The draft must exist, be a draft, and carry exactly the roster's assets. */
function draftAssets(tag) {
  const view = JSON.parse(gh(["release", "view", tag, "--json", "isDraft,assets"]));
  if (view.isDraft !== true) fail(`${tag} is not a draft — a published release is never re-pinned`);
  const names = view.assets.map((a) => a.name).sort();
  if (names.length !== ASSETS) fail(`${tag} carries ${names.length} assets, expected ${ASSETS}: ${names.join(", ")}`);
  return names;
}

/** SHA256SUMS parsed: name -> hash, refusing any name off the roster. */
function sums(tag, ver) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "ce-pin-"));
  gh(["release", "download", tag, "--pattern", "SHA256SUMS", "--dir", dir, "--clobber"]);
  const want = roster(ver);
  const got = {};
  for (const line of fs.readFileSync(path.join(dir, "SHA256SUMS"), "utf8").split("\n")) {
    if (!line.trim()) continue;
    const m = /^([0-9a-f]{64}) [ *](\S+)$/.exec(line);
    if (!m) fail(`SHA256SUMS line is not '<sha256>  <name>': ${line}`);
    if (!want[m[2]]) fail(`SHA256SUMS names an asset off the roster: ${m[2]}`);
    got[m[2]] = m[1];
  }
  const missing = Object.keys(want).filter((n) => !got[n]);
  if (missing.length) fail(`SHA256SUMS lacks: ${missing.join(", ")}`);
  fs.rmSync(dir, { recursive: true, force: true });
  return got;
}

/** Replace the one line `KEY="..."` in the manifest text; exactly one must exist. */
function setLine(text, key, value) {
  const re = new RegExp(`^${key}="[^"]*"$`, "m");
  const hits = text.match(new RegExp(re.source, "gm")) || [];
  if (hits.length !== 1) fail(`manifest has ${hits.length} lines for ${key}, expected 1`);
  return text.replace(re, `${key}="${value}"`);
}

/** A sibling script, inheriting stdio; a non-zero exit refuses by name. */
function sibling(script, args, env) {
  const r = spawnSync(process.execPath, [path.join(__dirname, script), ...args], {
    cwd: ROOT,
    stdio: "inherit",
    env: { ...process.env, ...env },
  });
  if (r.status !== 0) fail(`${script} exited ${r.status}`);
}

function main() {
  const ver = process.argv[2];
  if (!/^\d+\.\d+\.\d+$/.test(ver || "")) fail("usage: node scripts/pin_release.js <N.N.N> [--bless]");
  const tag = `v${ver}`;
  draftAssets(tag);
  const hashes = sums(tag, ver);

  const before = fs.readFileSync(MANIFEST, "utf8");
  const versionMoves = !before.includes(`CE_MANIFEST_VERSION="${ver}"`);
  let text = before;
  text = setLine(text, "CE_MANIFEST_VERSION", ver);
  text = setLine(text, "CE_BASE_URL", `https://github.com/skymanbp/CodeEraser/releases/download/${tag}`);
  for (const [name, key] of Object.entries(roster(ver))) text = setLine(text, key, hashes[name]);
  fs.writeFileSync(MANIFEST, text);

  // the proof is the diff, not the intent: every pin moves, plus the two
  // version lines when the version moves — nothing else may
  const numstat = execFileSync("git", ["diff", "--numstat", "--", "plugin/bin/manifest.env"], {
    cwd: ROOT,
    encoding: "utf8",
  }).trim();
  const [added, removed] = numstat ? numstat.split("\t").map(Number) : [0, 0];
  const expected = PINS + (versionMoves ? 2 : 0);
  if (added !== expected || removed !== expected) {
    fail(`manifest diff is +${added}/-${removed} lines, expected ${expected}/${expected} (git diff plugin/bin/manifest.env)`);
  }
  console.log(`pinned ${tag}: ${expected} lines moved in plugin/bin/manifest.env`);
  // the Homebrew formula and the winget manifests are projections of
  // the manifest just written — they move with it, in the same commit
  sibling("packaging.js", [], {});

  if (process.argv.includes("--bless")) {
    console.log("refreshing the docs-facts line (contracts/docs-facts.json ver:pin#v) …");
    const r = spawnSync("cargo", ["test", "--test", "it", "--", "facts_"], {
      cwd: path.join(ROOT, "cli"),
      stdio: "inherit",
      env: { ...process.env, CE_BLESS: "1" },
      shell: process.platform === "win32",
    });
    if (r.status !== 0) fail("the facts bless did not pass");
  } else {
    console.log("docs-facts line: CE_BLESS=1 cargo test --test it -- facts_   (or re-run with --bless)");
  }
}

main();
