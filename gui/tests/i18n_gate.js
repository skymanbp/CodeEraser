// The GUI's zh table gets the gate the CLI's has had since G3b:
// both-directions completeness. tr() resolves an unknown key to
// CE_I18N.en[key] → undefined, and applyStaticI18n assigns that to
// textContent — the literal string "undefined" on screen, silently.
// main_lang.rs reddens on a missing OR dead key; this leg asserts the
// same two properties over the real gui/ui files, harvesting usage
// from index.html (data-i18n / data-i18n-ph) and every screen's
// tr("...") calls.
//
// Usage: node gui/tests/i18n_gate.js   (exit 1 = a broken table)
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const root = path.join(__dirname, "..", "..");
const ui = (f) => fs.readFileSync(path.join(root, "gui/ui", f), "utf8");

// Load the REAL table under a stub sandbox (the lens gate's pattern).
const sandbox = { localStorage: { getItem: () => null, setItem() {} }, document: { querySelectorAll: () => [], getElementById: () => null } };
vm.createContext(sandbox);
vm.runInContext(ui("i18n.js"), sandbox);
const table = vm.runInContext("CE_I18N", sandbox);

// Harvest every key the UI actually asks for.
const used = new Set();
const html = ui("index.html");
for (const m of html.matchAll(/data-i18n(?:-ph)?="([^"]+)"/g)) used.add(m[1]);
// i18n.js included: axisName() consumes "axisNames" from inside the
// table's own file, and a harvest that skips it calls that key dead
for (const f of ["main.js", "structree.js", "trend.js", "candidates.js", "i18n.js"]) {
  for (const m of ui(f).matchAll(/\btr\(\s*"([^"]+)"/g)) used.add(m[1]);
}

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

// Direction 1: every used key exists in BOTH languages.
for (const lang of ["en", "zh"]) {
  const missing = [...used].filter((k) => !(k in table[lang]));
  say(missing.length === 0, `every used key exists in ${lang} (missing: ${missing.join(", ") || "none"})`);
}

// Direction 2: no dead keys — an entry nothing asks for is drift.
for (const lang of ["en", "zh"]) {
  const dead = Object.keys(table[lang]).filter((k) => !used.has(k));
  say(dead.length === 0, `no dead ${lang} keys (dead: ${dead.join(", ") || "none"})`);
}

// The two languages carry the same key set and the same value SHAPES
// (a function in one and a string in the other would explode only at
// call time, in one language).
const enKeys = Object.keys(table.en).sort().join(",");
const zhKeys = Object.keys(table.zh).sort().join(",");
say(enKeys === zhKeys, "en and zh carry the same key set");
const shapeSkew = Object.keys(table.en).filter((k) => typeof table.en[k] !== typeof table.zh[k]);
say(shapeSkew.length === 0, `value shapes agree (skew: ${shapeSkew.join(", ") || "none"})`);

// Non-vacuity: the harvest actually found the UI's keys.
say(used.size >= 30, `the harvest is real (${used.size} keys found)`);

process.exit(problems.length === 0 ? 0 : 1);
