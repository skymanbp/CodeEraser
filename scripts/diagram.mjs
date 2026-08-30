// The two bilingual diagrams — architecture and the judgment data flow —
// are authored as archify JSON IR under docs/diagrams/ and rendered
// here into the SVGs that README ×2 and the site hang (plan v2.21,
// architecture-diagram clause). archify (tt-a1i/archify, MIT) is not
// vendored: a tracked copy would enter the mention universe and the
// scan size arm. It is fetched at one pinned commit into the ignored
// cargo target directory, and every render refuses when the cache is
// absent or off-pin — a diagram nobody can regenerate is not a fact.
//
// The SVG is lifted out of archify's delivered HTML by the viewer's own
// "Download SVG" rules (assets/template.html serializeSvg, autoTheme),
// reimplemented over the static markup in scripts/diagram_svg.mjs: the
// semantic-class, theme-variable and archify-* keyframe rules of the
// page stylesheet, both variable sets resolved (dark default, light
// under prefers-color-scheme, svg[data-theme] forcing), a local()-only
// JetBrains Mono fallback and a var(--bg) background rect. One
// deliberate difference: <title> and <desc> stay the first children
// (the accessible name first, as SVG recommends and docs_lang.rs holds);
// the viewer puts its <style> before them.
//
// Usage: node scripts/diagram.mjs --fetch          populate the cache
//        node scripts/diagram.mjs [--check]        render, compare bytes
//        node scripts/diagram.mjs --write          render, write the SVGs
// Exit: 0 clean · 1 stale or failed · 2 cache absent / off-pin
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { extractSvg } from "./diagram_svg.mjs";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const PIN = "e1ac748f19cf805e44bf74fb93c796662152e273"; // tt-a1i/archify v2.15.0
const REMOTE = "https://github.com/tt-a1i/archify.git";
const CACHE = path.join(root, "cli", "target", "archify");
const BIN = path.join(CACHE, "archify", "bin", "archify.mjs");
const OUT = path.join(root, "cli", "target", "diagrams");
const DIAGRAMS = [
  ["architecture", "architecture"],
  ["judgment", "dataflow"],
];
const LANGS = ["en", "zh"];
const TWINS = ["docs/assets", "site/assets"];

function run(cmd, args, cwd) {
  const r = spawnSync(cmd, args, { cwd, encoding: "utf8", shell: false });
  if (r.error) throw r.error;
  if (r.status !== 0) {
    throw new Error(`${cmd} ${args.join(" ")} exited ${r.status}\n${r.stdout}${r.stderr}`);
  }
  return r.stdout;
}

const git = (args, cwd = CACHE) => run("git", args, cwd).trim();

function cacheHead() {
  if (!fs.existsSync(path.join(CACHE, ".git"))) return null;
  try {
    return git(["rev-parse", "HEAD"]);
  } catch {
    return null;
  }
}

function fetch() {
  fs.mkdirSync(CACHE, { recursive: true });
  if (!fs.existsSync(path.join(CACHE, ".git"))) {
    git(["init", "-q"]);
    git(["remote", "add", "origin", REMOTE]);
  }
  git(["fetch", "-q", "--depth", "1", "origin", PIN]);
  git(["checkout", "-q", "--detach", "FETCH_HEAD"]);
  const head = cacheHead();
  if (head !== PIN) throw new Error(`cache at ${head}, pinned ${PIN}`);
  console.log(`archify ${PIN.slice(0, 7)} ready at ${CACHE}`);
}

function requireCache() {
  const head = cacheHead();
  if (head === PIN && fs.existsSync(BIN)) return;
  console.error(
    `archify cache ${head ? `at ${head.slice(0, 7)}, pinned ${PIN.slice(0, 7)}` : "absent"} ` +
      `(${CACHE}) — run: node scripts/diagram.mjs --fetch`,
  );
  process.exit(2);
}

function deliver(name, type, lang) {
  const ir = path.join(root, "docs", "diagrams", `${name}.${lang}.json`);
  const html = path.join(OUT, `${name}.${lang}.html`);
  fs.mkdirSync(OUT, { recursive: true });
  // repository evidence (component `sources` checked at the pinned
  // revision) is an architecture-only feature of this archify pin
  const evidence = type === "architecture" ? ["--repo-root", root] : [];
  const args = ["deliver", type, ir, html, "--quality", "showcase", "--json", ...evidence];
  const receipt = JSON.parse(run("node", [BIN, ...args], path.dirname(BIN)));
  return { receipt, html: fs.readFileSync(html, "utf8") };
}

// ---- the check / write loop ----

const sha = (s) => createHash("sha256").update(s).digest("hex");

function main(argv) {
  if (argv.includes("--fetch")) return fetch();
  requireCache();
  const write = argv.includes("--write");
  let stale = 0;
  for (const [name, type] of DIAGRAMS) {
    for (const lang of LANGS) {
      const { receipt, html } = deliver(name, type, lang);
      const svg = extractSvg(html);
      const spec = receipt.specification?.sha256 ?? "?";
      const art = receipt.artifact?.sha256 ?? "?";
      console.log(`${name}.${lang}: spec ${spec.slice(0, 12)} artifact ${art.slice(0, 12)} svg ${sha(svg).slice(0, 12)}`);
      for (const dir of TWINS) {
        const target = path.join(root, dir, `${name}.${lang}.svg`);
        const current = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : null;
        if (current === svg) continue;
        if (write) {
          fs.writeFileSync(target, svg);
          console.log(`  wrote ${dir}/${name}.${lang}.svg`);
        } else {
          stale += 1;
          console.log(`  stale ${dir}/${name}.${lang}.svg (${current === null ? "absent" : "differs"}) — node scripts/diagram.mjs --write`);
        }
      }
    }
  }
  if (stale) process.exit(1);
}

main(process.argv.slice(2));
