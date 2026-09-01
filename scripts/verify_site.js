#!/usr/bin/env node
// Deploy verification for codeeraser.dev (docs/RELEASE.md 3): every
// page the site serves must be the blob this repository committed.
//
// Cloudflare splices exactly one analytics beacon into each HTML page
// at the edge, so "identical" cannot mean "identical bytes" and must
// not soften into "identical after removing anything script-shaped" —
// that reading would swallow a SECOND, real difference along with the
// first. The verdict here is narrow: the page carries exactly ONE
// injection, matched by the beacon's own host, and with that one
// occurrence removed it equals the committed blob byte for byte.
//
// Two things this script exists to remember, both learned the hard
// way: the tag arrives WITH the newline it sits on (strip only the
// tag and every page reads as one byte different), and the beacon
// carries attributes, so a pattern written as the bare string
// `<script type="module">` matches nothing and reports the whole site
// as changed — a red instrument, not a red site.

const { execFileSync } = require("child_process");
const path = require("path");

const ROOT = path.join(__dirname, "..");
const BASE = "https://codeeraser.dev";

/** Committed page -> the path it is served at. */
const PAGES = [
  ["site/index.html", "/"],
  ["site/how/index.html", "/how/"],
  ["site/stack/index.html", "/stack/"],
  ["site/bench/index.html", "/bench/"],
  ["site/zh/index.html", "/zh/"],
  ["site/zh/how/index.html", "/zh/how/"],
  ["site/zh/stack/index.html", "/zh/stack/"],
  ["site/zh/bench/index.html", "/zh/bench/"],
];

const BEACON =
  /<script[^>]*static\.cloudflareinsights\.com[^>]*>\s*<\/script>\n?/g;

// Cloudflare answers a default fetch agent with 403; say who we are.
const UA = "Mozilla/5.0 (compatible; codeeraser-verify-site)";

function committed(rel) {
  return execFileSync("git", ["-C", ROOT, "show", `HEAD:${rel}`], {
    encoding: "buffer",
    maxBuffer: 1 << 26,
  });
}

async function served(urlPath) {
  const res = await fetch(BASE + urlPath, { headers: { "user-agent": UA } });
  if (!res.ok) throw new Error(`${urlPath}: HTTP ${res.status}`);
  return Buffer.from(await res.arrayBuffer());
}

/** Where two buffers first differ, with a little of each side. */
function firstDifference(a, b) {
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i += 1) {
    if (a[i] !== b[i]) {
      const lo = Math.max(0, i - 60);
      return `\n  served: ${a.subarray(lo, i + 60)}\n  blob  : ${b.subarray(lo, i + 60)}`;
    }
  }
  return `\n  one is a prefix of the other (served ${a.length}, blob ${b.length})`;
}

async function verdict(rel, urlPath) {
  const blob = committed(rel);
  const live = await served(urlPath);
  const hits = live.toString("utf8").match(BEACON) || [];
  const stripped = Buffer.from(live.toString("utf8").replace(BEACON, ""));

  if (hits.length === 1 && stripped.equals(blob)) {
    return [true, `OK    ${urlPath} — one beacon stripped, then identical`];
  }
  if (hits.length === 0 && live.equals(blob)) {
    return [true, `OK    ${urlPath} — byte-identical, no beacon`];
  }
  return [
    false,
    `DIFF  ${urlPath} — beacons=${hits.length} served=${live.length} ` +
      `blob=${blob.length}${firstDifference(stripped, blob)}`,
  ];
}

async function main() {
  let bad = 0;
  for (const [rel, urlPath] of PAGES) {
    const [ok, line] = await verdict(rel, urlPath);
    if (!ok) bad += 1;
    console.log(line);
  }
  const good = PAGES.length - bad;
  console.log(`${good}/${PAGES.length} pages match the committed blob`);
  process.exit(bad ? 1 : 0);
}

main().catch((err) => {
  console.error(`verify_site: ${err.message}`);
  process.exit(1);
});
