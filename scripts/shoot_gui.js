// Regenerate the three GUI screenshots the website shows by running the
// product, the stance the homepage terminal block already takes
// (cli/tests/it/site_roast.rs): a picture on the site is a REPORT.
// The three PNGs were posed by hand on 2026-08-22 and went stale four
// ways at once in silence, because nothing could re-derive them; the
// gate that holds them, cli/tests/it/site_screenshots.rs, names all four.
// The renderer is headless Edge — the SAME engine the shipped app draws
// through, since Tauri on Windows is WebView2 — so these are the
// product's own pixels. Only the `invoke` bridge has no browser
// equivalent; it is stubbed with the documents the CLI produced, the
// same ones `gui/src-tauri/src/commands.rs` hands the webview.
//
// Usage: node scripts/shoot_gui.js --out site/assets
//        [--ce <path>] [--root <repo>] [--browser <exe>]
//        [--save <dir>] [--reports <dir>]  (`ce join` costs minutes:
// --save keeps a run's three documents, --reports shoots from them again.)
"use strict";

const { Devtools, devtoolsUrl } = require("./cdp.js");
const { spawn, execFileSync } = require("child_process");
const crypto = require("crypto");
const fs = require("fs");
const http = require("http");
const os = require("os");
const path = require("path");

const REPO = path.resolve(__dirname, "..");
const UI = path.join(REPO, "gui", "ui");
const SITE = path.join(REPO, "site", "assets");

// Every shot the site carries, in homepage order. `act` runs in the
// page; `until` is polled until it answers true.
const SHOTS = [
  {
    name: "gui-structure",
    act: `document.getElementById("scan").click()`,
    until: `!document.getElementById("summary").hidden
            && document.querySelectorAll("#treemap rect").length > 20`,
  },
  {
    name: "gui-tree",
    act: `document.querySelector('#vswitch button[data-v="tree"]').click()`,
    until: `!document.getElementById("structree").hidden
            && document.querySelectorAll("#structree .trow").length > 5`,
  },
  {
    name: "gui-candidates",
    act: `document.querySelector('[data-tab="candidates"]').click();
          document.getElementById("cand-load").click()`,
    until: `document.getElementById("cand-list").children.length > 0`,
  },
];

// The window the site's figures are cropped to — the shipped app's own
// default, so a reader comparing picture to screen sees one shape.
const VIEW = { width: 1424, height: 892, deviceScaleFactor: 1, mobile: false };

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".png": "image/png",
};

function arg(flag, fallback) {
  const i = process.argv.indexOf(flag);
  return i > 0 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
}

// Edge first, because it IS the engine the app ships on; Chrome speaks
// the same protocol and is the fallback. All three platforms are probed:
// the gate that sends people here runs on Linux and macOS too.
function browser() {
  const named = arg("--browser", null);
  if (named) return named;
  const roots = [process.env["PROGRAMFILES(X86)"], process.env.PROGRAMFILES, process.env.LOCALAPPDATA];
  const rel = [
    ["Microsoft", "Edge", "Application", "msedge.exe"],
    ["Google", "Chrome", "Application", "chrome.exe"],
  ];
  const candidates = [];
  for (const r of roots.filter(Boolean)) {
    for (const p of rel) candidates.push(path.join(r, ...p));
  }
  candidates.push(
    "/usr/bin/microsoft-edge",
    "/usr/bin/microsoft-edge-stable",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  );
  const found = candidates.find((c) => fs.existsSync(c));
  if (found) return found;
  throw new Error("no Edge or Chrome found — pass --browser <path>");
}

// The three documents the three screens render. Captured from the CLI
// so the pictures show a judgment anyone can reproduce with the same
// three commands, printed below for exactly that reason. `--save` is
// `--reports`' twin: a re-shoot that only changes framing should not
// re-judge a repository that has not moved.
function reports(root) {
  const dir = arg("--reports", null);
  const read = (at, f) => JSON.parse(fs.readFileSync(path.join(at, f), "utf8"));
  if (dir) {
    return {
      structure: read(dir, "structure.json"),
      join: read(dir, "join.json"),
      dedup: read(dir, "dedup.json"),
    };
  }
  const ce = arg("--ce", "ce");
  const run = (args) => {
    console.log(`  $ ${path.basename(ce)} ${args.join(" ")}`);
    return JSON.parse(execFileSync(ce, [...args, "--format", "json"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 1 << 28,
    }));
  };
  const docs = {
    structure: run(["structure"]),
    join: run(["join", "--days", "14"]),
    dedup: run(["dedup"]),
  };
  const save = arg("--save", null);
  if (save) {
    fs.mkdirSync(save, { recursive: true });
    for (const [name, doc] of Object.entries(docs)) {
      fs.writeFileSync(path.join(save, `${name}.json`), JSON.stringify(doc));
    }
  }
  return docs;
}

// Serve gui/ui over http, not file://: the shell keeps the typed root in
// localStorage, which a file: origin refuses — so the app would boot into
// a state users never see.
function serve() {
  const server = http.createServer((req, res) => {
    const rel = decodeURIComponent(req.url.split("?")[0]).replace(/^\/+/, "") || "index.html";
    const file = path.join(UI, rel);
    if (!file.startsWith(UI) || !fs.existsSync(file)) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { "content-type": MIME[path.extname(file)] || "application/octet-stream" });
    res.end(fs.readFileSync(file));
  });
  return new Promise((ok) => server.listen(0, "127.0.0.1", () => ok(server)));
}

/// The webview bridge, installed before any page script runs — the
/// shell reads `default_root` during boot, so a stub added after
/// navigation is already too late. Anything the harness has no answer
/// for REJECTS by name: a silent undefined photographs half a screen.
function bridge(root, docs) {
  return `(() => {
    const ROOT = ${JSON.stringify(root)};
    const DOCS = ${JSON.stringify(docs)};
    const answers = {
      default_root: () => ROOT,
      resolve_root: () => ({ root: ROOT, ascended: false }),
      structure_report: () => DOCS.structure,
      join_report: () => DOCS.join,
      dedup_report: () => DOCS.dedup,
    };
    window.__TAURI__ = {
      core: {
        invoke: (cmd) => answers[cmd]
          ? Promise.resolve(answers[cmd]())
          : Promise.reject(new Error("screenshot harness has no answer for " + cmd)),
      },
      event: { listen: () => Promise.resolve(() => {}) },
    };
  })()`;
}

/// Attach to the browser, install the bridge, and load the shell.
async function attach(profile, origin, root, docs) {
  const dt = await devtoolsUrl(profile);
  const list = JSON.parse(await (await fetch(`${dt.http}/json/list`)).text());
  const page = list.find((t) => t.type === "page");
  const cdp = await Devtools.open(page.webSocketDebuggerUrl);
  await cdp.send("Page.enable");
  await cdp.send("Runtime.enable");
  await cdp.send("Emulation.setDeviceMetricsOverride", VIEW);
  // Reduced motion, because the app already answers it (style.css:
  // `button { transition: none }`). Every shot clicks a button, whose
  // background transitions over 120ms; a capture two frames later
  // landed at an arbitrary point along it, and gui-tree came out with
  // a different digest on each of three runs off ONE report set.
  await cdp.send("Emulation.setEmulatedMedia", {
    features: [{ name: "prefers-reduced-motion", value: "reduce" }],
  });
  await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: bridge(root, docs) });
  await cdp.send("Page.navigate", { url: origin });
  await cdp.until(`document.readyState === "complete" && window.tr`, "the shell to boot");
  return cdp;
}

/// Drive each screen and photograph it.
async function capture(cdp, out) {
  for (const shot of SHOTS) {
    await cdp.eval(shot.act);
    await cdp.until(shot.until, shot.name);
    // two frames for layout and paint to land (transitions are off)
    await cdp.eval(`new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)))`);
    const { data } = await cdp.send("Page.captureScreenshot", { format: "png" });
    const file = path.join(out, `${shot.name}.png`);
    fs.writeFileSync(file, Buffer.from(data, "base64"));
    console.log(`  ${path.relative(REPO, file)}  ${VIEW.width}x${VIEW.height}` +
      `  ${(fs.statSync(file).size / 1024).toFixed(0)}KB`);
  }
}

/// The receipt: which report schemas these pictures show, and which
/// bytes were actually shot. Ancestry alone would not have caught the
/// bug that started this — the candidates screen showed
/// `ce.join-report/0.1.0` through two schema bumps without `gui/ui`
/// changing once. The digests tie the receipt to the pixels; without
/// them a schema bump could be answered by editing three strings here
/// while the old picture stayed on the page.
function receipt(docs, out) {
  const file = path.join(REPO, "contracts", "gui-shots.json");
  if (out !== SITE) {
    console.log(`  receipt untouched — ${path.relative(REPO, out)} is not what the site serves`);
    return;
  }
  const shots = {};
  for (const shot of SHOTS) {
    const png = fs.readFileSync(path.join(out, `${shot.name}.png`));
    shots[`${shot.name}.png`] = crypto.createHash("sha256").update(png).digest("hex");
  }
  const body = {
    note: "Written by scripts/shoot_gui.js; gated by cli/tests/it/site_screenshots.rs.",
    window: [VIEW.width, VIEW.height],
    shots,
    schemas: {
      structure: docs.structure.schema,
      join: docs.join.schema,
      dedup: docs.dedup.schema,
    },
  };
  fs.writeFileSync(file, JSON.stringify(body, null, 2) + "\n");
  console.log(`  ${path.relative(REPO, file)}`);
}

async function main() {
  const out = path.resolve(arg("--out", SITE));
  const root = path.resolve(arg("--root", REPO));
  fs.mkdirSync(out, { recursive: true });

  console.log(`judging ${root}`);
  const docs = reports(root);
  console.log(`  score ${docs.structure.score}/${docs.structure.scoreScale}` +
    `, ${docs.structure.schema}, ${docs.join.schema}, ${docs.dedup.schema}`);

  const server = await serve();
  const origin = `http://127.0.0.1:${server.address().port}/index.html`;
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "ce-shoot-"));
  const proc = spawn(browser(), [
    "--headless=new",
    "--remote-debugging-port=0",
    `--user-data-dir=${profile}`,
    `--window-size=${VIEW.width},${VIEW.height}`,
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--hide-scrollbars",
    "about:blank",
  ], { stdio: "ignore" });

  try {
    await capture(await attach(profile, origin, root, docs), out);
    // after the pictures, so a failed shoot leaves no receipt claiming
    // pictures it did not take
    receipt(docs, out);
  } finally {
    // Cleanup must never throw: the browser keeps its profile mapped for
    // a moment after the kill, and an EBUSY raised here would REPLACE the
    // real failure — which is exactly what hid the first shot timeout.
    proc.kill();
    server.close();
    for (let i = 0; i < 20; i++) {
      try {
        fs.rmSync(profile, { recursive: true, force: true });
        break;
      } catch {
        await new Promise((r) => setTimeout(r, 100));
      }
    }
  }
}

main().catch((e) => {
  console.error(String(e.message || e));
  process.exit(1);
});
