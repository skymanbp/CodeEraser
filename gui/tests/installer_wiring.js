// The installer's Claude Code wiring, gated (K step 10). The passive
// lane once died for three days on exactly this class: 9f86d58 moved
// the marketplace manifest to the repo root, the old plugin/-pointing
// registration kept resolving to a cache miss, and no leg noticed —
// the wiring is exercised only when a human runs the NSIS installer
// on a machine with Claude Code, which CI never is. This gate pins
// the STATIC half of the chain: every name and path the wiring
// script mentions must resolve against the repository it ships from,
// so a rename or move reddens the push that made it instead of the
// install three days later.
//
// Usage: node gui/tests/installer_wiring.js   (exit 1 = broken wiring)
"use strict";
const fs = require("fs");
const path = require("path");

const root = path.join(__dirname, "..", "..");
const read = (p) => fs.readFileSync(path.join(root, p), "utf8");

const problems = [];
function say(ok, what) {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
}

// 1. The bundler actually includes the hook script it names.
const conf = JSON.parse(read("gui/src-tauri/tauri.conf.json"));
const hooksRel = conf.bundle.windows.nsis.installerHooks;
const hooksPath = path.join(root, "gui/src-tauri", hooksRel);
say(fs.existsSync(hooksPath), `installerHooks resolves (${hooksRel})`);
const nsh = fs.readFileSync(hooksPath, "utf8");

// 2. Marker symmetry: the wired marker is written by POSTINSTALL and
// keyed on + deleted by PREUNINSTALL — an asymmetric rename leaves
// either an unremovable marketplace or an uninstall that removes a
// registration this installer never made.
const macro = (name) => {
  const at = nsh.indexOf(`!macro ${name}`);
  const end = nsh.indexOf("!macroend", at);
  return at >= 0 && end > at ? nsh.slice(at, end) : "";
};
const marker = "claude-plugin-wired";
say(macro("NSIS_HOOK_POSTINSTALL").includes(marker), "POSTINSTALL writes the marker");
const pre = macro("NSIS_HOOK_PREUNINSTALL");
say(pre.includes(`IfFileExists "$INSTDIR\\${marker}"`), "PREUNINSTALL keys on the marker");
say(pre.includes(`Delete "$INSTDIR\\${marker}"`), "PREUNINSTALL deletes the marker");

// 3. The registration slug names THIS repository, and the manifest it
// will resolve sits at the repo ROOT (the 9f86d58 lesson: plugin/ was
// a silent cache miss). WHOLE-TOKEN equality, not includes(): a
// prefix test waved "skymanbp/CodeEraser-plugin" through in this
// gate's own counterfactual run — the wrong-slug case it exists for.
// … and the file spells the slug three times — the exec line plus
// the comments' manual-recovery command (with prose punctuation), and
// the FIRST bare match is even the exit-code legend's "add failed".
// So: every slug-shaped mention (owner/repo carries a slash),
// punctuation stripped, deduped — the SET must be exactly this
// repository. A comment steering the manual-recovery user to a stale
// slug is the same defect as the exec line carrying it.
const slugs = [
  ...new Set(
    [...nsh.matchAll(/marketplace add (\S+)/g)]
      .map((x) => x[1].replace(/[,.)]+$/, ""))
      .filter((x) => x.includes("/"))
  ),
];
say(
  slugs.length === 1 && slugs[0] === "skymanbp/CodeEraser",
  `every slug mention is this repository (${slugs.join(", ") || "none"})`
);
const mp = JSON.parse(read(".claude-plugin/marketplace.json"));
say(mp.name === "codeeraser", `root manifest is the marketplace (name=${mp.name})`);

// 4. The install target's two halves resolve: plugin@marketplace.
const m = nsh.match(/plugin install ([a-z-]+)@([a-z-]+)/);
say(!!m, "an install target exists");
if (m) {
  say(m[2] === mp.name, `target marketplace ${m[2]} = manifest name`);
  const plugin = (mp.plugins || []).find((p) => p.name === m[1]);
  say(!!plugin, `target plugin ${m[1]} exists in the manifest`);
  if (plugin) {
    const src = path.join(root, plugin.source);
    say(fs.existsSync(src), `plugin source resolves (${plugin.source})`);
  }
}

process.exit(problems.length === 0 ? 0 : 1);
