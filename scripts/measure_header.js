// Measure the GUI header in the engine the app ships on (headless Edge
// = WebView2), in both languages: the width below which the header can
// no longer seat its children even with every squeezable column — the
// root field at its min-width, the echo and the status column at zero
// — at its floor, so the tab strip and the language button start to
// clip. That FLOOR is the number gui/ui/style.css pins for the strip's
// wrap breakpoint; the batch-9 proposal that asked for the wrap asked
// for exactly this measurement before any number was pinned (plan
// v2.29 step 9, O25). The ladder beside it shows what a reader sees at
// each width. Nothing here judges a repository: with --reports the
// bridge answers with saved documents (the status line then reads as
// it does in the product), without it the shell boots on the repo path
// and no reports.
//
// Usage: node scripts/measure_header.js [--browser <exe>] [--reports <dir>]
//        [--widths 860,900,...]   (the ladder; default below)
"use strict";

const { REPO, VIEW, arg, attach, launch, reports, serve, teardown } = require("./shoot_gui.js");
const fs = require("fs");
const os = require("os");
const path = require("path");

// From the app's own minimum window (tauri.conf.json minWidth 860) up
// to the site's figure width.
const LADDER = "860,900,940,980,1020,1060,1100,1140,1180,1220,1280,1424";

// What a reader sees at the current width: how many rows the header
// took (items share a row when their vertical centres coincide —
// `align-items: center` puts children of different heights at
// different tops), whether children spill past the header, the widths
// the squeezable columns kept, and which children clip their own
// content — the root field and the status column are built to
// (scrolling input, ellipsis), so only the others count as clipping.
const PROBE = `(() => {
  const h = document.querySelector("header");
  const kids = [...h.children];
  const rect = (e) => e.getBoundingClientRect();
  const lines = (els) => new Set([...els].map((e) => Math.round(rect(e).top + rect(e).height / 2))).size;
  const w = (id) => Math.round(rect(document.getElementById(id)).width);
  return {
    rows: lines(kids),
    strip: lines(document.getElementById("tabs").children),
    spill: h.scrollWidth > h.clientWidth + 1,
    tabs: w("tabs"), root: w("root"), status: w("status"),
    clipped: kids.filter((e) => !["root", "status", "rootecho"].includes(e.id))
      .filter((e) => e.scrollWidth > e.clientWidth + 1).map((e) => e.id || e.tagName.toLowerCase()),
  };
})()`;

async function settle(cdp) {
  await cdp.eval(`new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))`);
}

async function probeAt(cdp, width) {
  await cdp.send("Emulation.setDeviceMetricsOverride", { ...VIEW, width });
  await settle(cdp);
  return { width, ...(await cdp.eval(PROBE)) };
}

function row(p) {
  return `  ${String(p.width).padEnd(6)} ${String(p.rows).padEnd(5)} ${String(p.strip).padEnd(6)} ${(p.spill ? "yes" : "no").padEnd(6)}` +
    ` ${String(p.tabs).padEnd(5)} ${String(p.root).padEnd(5)} ${String(p.status).padEnd(7)} ${p.clipped.join(",") || "-"}`;
}

/// The smallest viewport width at which nothing spills or clips on a
/// one-row header, by bisection over [lo, hi]: the width just below
/// it is where the header needs the wrap.
async function floor(cdp, lo, hi) {
  const fits = (p) => !p.spill && p.clipped.length === 0;
  if (!fits(await probeAt(cdp, hi))) throw new Error(`the header does not fit even at ${hi}px`);
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1;
    if (fits(await probeAt(cdp, mid))) hi = mid;
    else lo = mid;
  }
  return hi;
}

async function ladder(cdp, lang) {
  console.log(`${lang}:`);
  console.log("  width  rows  strip  spill  tabs  root  status  clipped");
  for (const width of arg("--widths", LADDER).split(",").map(Number)) {
    console.log(row(await probeAt(cdp, width)));
  }
  const fit = await floor(cdp, 600, VIEW.width);
  console.log(`  fits from ${fit}px; at ${fit - 1}px: ${row(await probeAt(cdp, fit - 1)).trim()}`);
  return fit;
}

async function main() {
  const docs = arg("--reports", null) ? reports(REPO) : {};
  const server = await serve();
  const origin = `http://127.0.0.1:${server.address().port}/index.html`;
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "ce-measure-"));
  const proc = launch(profile);
  try {
    const cdp = await attach(profile, origin, REPO, docs);
    const en = await ladder(cdp, "en");
    const label = `document.querySelector('[data-tab="structure"]').textContent`;
    const before = await cdp.eval(label);
    await cdp.eval(`document.getElementById("lang").click()`);
    await cdp.until(`${label} !== ${JSON.stringify(before)}`, "the language to switch");
    const zh = await ladder(cdp, "zh");
    console.log(`floor: en ${en}px, zh ${zh}px — wrap below max(en, zh) = @media (max-width: ${Math.max(en, zh) - 1}px)`);
  } finally {
    await teardown(proc, server, profile);
  }
}

main().catch((e) => {
  console.error(String(e.message || e));
  process.exit(1);
});
