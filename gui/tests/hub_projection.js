// K44 (plan v2.17 L round, sealed criterion §7): the diagnostics
// hub renders a report's row arrays generically, keeping the first
// five scalar columns of row 0 — so a 0.3.0 advisory row of the
// deadcode report must keep its `symbol` (and `name`) through that
// projection, or the symbol-level face shows a file with no symbol.
// serde_json emits the row's keys in BTreeMap order (alphabetical:
// code, line, name, symbol, why — the Rust half of the pin is
// cli/tests/it/deadcode_e2e.rs), which is the order this projection
// sees; the leg drives the REAL gui/ui/reports.js under DOM stubs and
// reads the rendered header cells.
//
// Usage: node gui/tests/hub_projection.js   (exit 1 = a lost column)
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const root = path.join(__dirname, "..", "..");

// Just enough DOM for the module's boot IIFE (a select it fills, two
// listeners) — every sink is a no-op, hubTable is called directly.
const stubEl = () => ({
  addEventListener() {},
  set innerHTML(_) {},
  set hidden(_) {},
  set disabled(_) {},
});
const sandbox = {
  Object,
  Array,
  String,
  Number,
  document: { getElementById: stubEl },
  $: stubEl,
  i18nRefreshers: [],
  tr: (k, ...a) => `${k}(${a.join(",")})`,
  esc: (s) => String(s),
  posInt: (v) => v,
  invoke: async () => ({}),
  setStatus() {},
};
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(path.join(root, "gui/ui/reports.js"), "utf8"), sandbox);

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

// One advisory row exactly as ce.deadcode-report/0.3.0 emits it.
const row = {
  code: "public_unmentioned",
  line: 84,
  name: "cli/src/config.rs",
  symbol: "DedupCfg",
  why: "no other file spells this exported name",
};
const headers = (html) => [...html.matchAll(/<th[^>]*>([^<]*)<\/th>/g)].map((m) => m[1]);
const cols = headers(sandbox.hubTable(["unmentioned", [row]]));
say(cols.includes("symbol") && cols.includes("name"), `symbol and name survive the projection (${cols.join(",")})`);
say(cols.join(",") === "code,line,name,symbol,why", "all five advisory columns render, in document order");
const body = sandbox.hubTable(["unmentioned", [row]]);
say(body.includes(">DedupCfg<"), "the symbol cell carries the name");

// Non-vacuity: the projection IS a cut — a sixth scalar column is
// dropped, so the five-column survival above is not a vacuous pass
// on a renderer that keeps everything.
const wide = { ...row, extra: "sixth" };
say(!headers(sandbox.hubTable(["wide", [wide]])).includes("extra"), "the projection really cuts at five columns");

process.exit(problems.length === 0 ? 0 : 1);
