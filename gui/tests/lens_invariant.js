// The structure screen's two lenses share one fold rule, and this is
// the gate on what that rule is allowed to hide. It loads the REAL
// gui/ui/structree.js under DOM stubs and walks it over a fixture
// shaped to hit every chain boundary (contracts/fixtures/structure/
// gui-lens.json), because gui/ui/*.js is outside the scanner's
// language set and had no gate of its own.
//
// The invariant: a fold may swallow a pass-through directory, never a
// directory with findings. Folding on SHAPE alone hid 8 findings
// across 4 directories of this very repository — rendered by neither
// lens, and with no row or rect carrying their id, unreachable by
// click as well (review 2026-08-19).
//
// Usage: node gui/tests/lens_invariant.js   (exit 1 = a broken lens)
"use strict";
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const root = path.join(__dirname, "..", "..");
const report = JSON.parse(
  fs.readFileSync(path.join(root, "contracts/fixtures/structure/gui-lens.json"), "utf8"),
);

// Just enough DOM for the module's top-level listener wiring and for
// drawTree() to run: every sink is a no-op, the assertions read the
// module's own state instead of any rendered markup.
const stubEl = () => ({
  addEventListener() {},
  querySelectorAll: () => [],
  querySelector: () => null,
  classList: { toggle() {}, add() {}, remove() {} },
  toggleAttribute() {},
  set innerHTML(_) {},
  set hidden(_) {},
  forEach() {},
});

const AXES = ["geometry", "naming", "mixing", "misplaced", "docs", "stale", "redundancy"];
const sandbox = {
  Math,
  Set,
  Number,
  String,
  JSON,
  console,
  localStorage: { getItem: () => null, setItem() {} },
  document: { getElementById: stubEl, querySelectorAll: () => [] },
  $: stubEl,
  tr: (k) => (k === "axisNames" ? AXES : k),
  axisName: (c) => AXES[c] ?? String(c),
  esc: (s) => String(s),
  heat: () => "#000",
  LEGEND_STOPS: ["s0", "s1", "s2", "s3", "s4"],
  drawTreemap() {},
  showDetail() {},
  report,
  selId: 0,
};
sandbox.children = report.tree.map(() => []);
report.tree.forEach((n, i) => {
  if (i > 0) sandbox.children[n.parent].push(i);
});
vm.createContext(sandbox);
vm.runInContext(fs.readFileSync(path.join(root, "gui/ui/structree.js"), "utf8"), sandbox);

// Walk the fold exactly as both lenses do: each row names its chain's
// TAIL and hides the rest of the path.
const named = new Set();
const hidden = new Set();
(function walk(id) {
  const chain = sandbox.foldChain(id);
  const tail = chain[chain.length - 1];
  named.add(tail);
  chain.slice(0, -1).forEach((i) => hidden.add(i));
  for (const kid of sandbox.children[tail]) walk(kid);
})(0);

const problems = [];
const say = (ok, what) => {
  console.log(`${ok ? "ok  " : "FAIL"} ${what}`);
  if (!ok) problems.push(what);
};

const lost = report.tree
  .map((n, i) => ({ i, n }))
  .filter(({ i, n }) => n.axes.length > 0 && !named.has(i));
say(lost.length === 0, `every findings-bearing directory is named by a row (${lost.length} lost)`);
lost.forEach(({ i, n }) => console.log(`       lost: ${i} ${n.name} axes=${JSON.stringify(n.axes)}`));

say(hidden.size > 0, `empty pass-through directories still fold (${hidden.size} folded)`);

// Non-emptiness (the F16 discipline): a gate that cannot fail is not
// a gate. The rule that SHIPPED folded through any single-child node,
// findings or not — it must lose at least one finding on this
// fixture, or the assertion above is passing on a tree with no
// hazard in it.
const shapeOnlyNamed = new Set();
(function shapeWalk(id) {
  let cur = id;
  while (sandbox.children[cur].length === 1) cur = sandbox.children[cur][0];
  shapeOnlyNamed.add(cur);
  for (const kid of sandbox.children[cur]) shapeWalk(kid);
})(0);
const shapeOnlyLost = report.tree.filter((n, i) => n.axes.length > 0 && !shapeOnlyNamed.has(i)).length;
say(shapeOnlyLost > 0, `the fixture exercises the hazard (shape-only fold loses ${shapeOnlyLost})`);

// drawTree() seeds default expansion and computes the colour scale;
// running it also proves the module survives a full pass.
sandbox.drawTree();
const rootChain = sandbox.foldChain(0);
say(
  rootChain.every((i) => !named.has(i) || i === rootChain[rootChain.length - 1]),
  "the root chain resolves to exactly one row",
);

// Default expansion is keyed on the chain HEAD — the node whose depth
// sets the row's indent. Seeding by depth and testing the TAIL left
// deep chains shut against the stated "three levels open" contract.
const headOf = new Map();
(function heads(id) {
  const chain = sandbox.foldChain(id);
  headOf.set(chain[chain.length - 1], chain[0]);
  for (const kid of sandbox.children[chain[chain.length - 1]]) heads(kid);
})(0);
// `let` at a script's top level lands in the context's lexical scope,
// not on the sandbox object — read it by evaluating in that context.
const expanded = vm.runInContext("expanded", sandbox);
const shutShallow = [...headOf].filter(
  ([tail, head]) =>
    report.tree[head].depth < 3 && sandbox.children[tail].length > 0 && !expanded.has(head),
);
say(shutShallow.length === 0, "rows whose head sits above level 3 open by default");

process.exit(problems.length === 0 ? 0 : 1);
