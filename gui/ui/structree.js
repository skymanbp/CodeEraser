// CodeEraser GUI — structure tree view (M8 polish). A sibling lens on
// the SAME report document the treemap renders: nothing judged here,
// nothing measured here. Where the treemap answers "how big / how
// hot", the tree answers "where exactly": each row rolls its
// subtree's findings up into its heat rail (log-ranked against the
// report's own range, so the rail is a gradient, not a wall) and
// names its own findings as inline axis tags. Shares main.js globals
// ($, report, children, selId, showDetail, drawTreemap, heat, esc).
"use strict";

let treeView = localStorage.getItem("ce-structure-view") === "tree";
let expanded = null; // ids whose children are visible
let lastTree = null; // reset expansion when a new report arrives

// The one structure repaint entry (main.js render/resize call this):
// picks the active lens, keeps the switch buttons in sync. The tree
// is HTML flow — a resize repaint is harmless (only scroll resets).
function redrawStructure() {
  $("vswitch").hidden = !report;
  $("vswitch").querySelectorAll("button").forEach((b) => {
    b.classList.toggle("on", (b.dataset.v === "tree") === treeView);
  });
  // toggleAttribute, not .hidden: SVG elements have no hidden IDL
  // property, so a bare assignment never reaches the [hidden] CSS
  $("treemap").toggleAttribute("hidden", treeView);
  $("structree").toggleAttribute("hidden", !treeView);
  if (treeView) drawTree();
  else drawTreemap();
}

// Subtree rollup, leaf-to-root over the report's parent-before-child
// order (the weights() trick) — used for findings AND file counts.
function rollup(get) {
  const r = report.tree.map(get);
  for (let i = report.tree.length - 1; i > 0; i--) {
    r[report.tree[i].parent] += r[i];
  }
  return r;
}

// The unary chain one row (or one treemap strip) stands for. BOTH
// lenses fold through this, so they name the same places the same way.
//
// The fold only swallows nodes with NOTHING OF THEIR OWN to say: it
// exists to stop empty pass-through levels burning a band each, and a
// directory carrying findings is not an empty level. Folding on shape
// alone hid 8 findings across 4 directories on this very repository
// (cli/src/{dedup,graph,scan}, contracts) — rendered nowhere, and with
// no row or rect carrying their id, unreachable by click as well.
function foldChain(id) {
  const path = [id];
  let cur = id;
  while (children[cur].length === 1 && !report.tree[cur].axes.length) {
    cur = children[cur][0];
    path.push(cur);
  }
  return path;
}

function drawTree() {
  if (lastTree !== report.tree) {
    lastTree = report.tree;
    // three levels open by default — keyed on the chain HEAD, which is
    // the node whose depth sets the row's indent (keying the seed by
    // depth and the test by the chain's TAIL left deep chains shut)
    expanded = new Set(report.tree.flatMap((n, i) => (n.depth < 3 ? [i] : [])));
  }
  const ctx = {
    roll: rollup((n) => n.axes.length),
    filesBelow: rollup((n) => n.files),
  };
  // A LOOP, not Math.max(...roll): the spread passes one argument per
  // directory and the core will judge up to structNodeCap of them,
  // which throws RangeError long before that — blanking the panel with
  // no status line, after localStorage had already stored "tree".
  //
  // The root chain is excluded for the same reason railColor exempts
  // it: its rollup IS the grand total, so letting it set the scale
  // would flatten every real row onto the cold end.
  const top = new Set(foldChain(0));
  ctx.mx = 1;
  for (let i = 0; i < ctx.roll.length; i++) {
    if (!top.has(i)) ctx.mx = Math.max(ctx.mx, ctx.roll[i]);
  }
  // a sticky header names the numeric columns once, so the data rows
  // carry bare tabular digits (no per-row unit words, no Σ sigils)
  const out = [
    `<div class="trow thead"><span class="tw"></span>` +
      `<span class="tname">${tr("thDir")}</span><span class="fchip">${tr("thOwn")}</span>` +
      `<span class="tsub">${tr("thSub")}</span><span class="tmeta">${tr("thFiles")}</span></div>`,
  ];
  walkRows(0, ctx, out);
  $("structree").innerHTML = out.join("");
}

// One row per rendered node; empty unary chains (core/app/CE …) fold
// into a single row exactly like the treemap's strips.
function walkRows(id, ctx, out) {
  const path = foldChain(id);
  const tail = path[path.length - 1];
  const label = path
    .map((i) => report.tree[i].name.split("/").pop() || ".")
    .join("/");
  out.push(rowHtml(tail, path[0], label, ctx));
  if (expanded.has(path[0])) {
    for (const k of children[tail]) walkRows(k, ctx, out);
  }
}

// Rail color: log-ranked against the report's own maximum, so the
// hottest branch is unique and the low end stays separable (a fixed
// 0..4 clamp painted most rows the same top stop). The root is
// exempt — its rollup is the grand total the summary strip already
// shows, and it would always be the hottest row on screen. Keyed on
// the chain HEAD: a single-child root folds away, and testing the
// tail then handed the top row exactly the heat this prevents.
function railColor(tail, head, ctx) {
  const r = ctx.roll[tail];
  if (head === 0) return "transparent";
  if (!r) return LEGEND_STOPS[0];
  return LEGEND_STOPS[1 + Math.floor((3 * Math.log1p(r)) / Math.log1p(ctx.mx))];
}

function rowHtml(tail, head, label, ctx) {
  const n = report.tree[tail];
  const own = n.axes.length;
  const twirl = children[tail].length ? (expanded.has(head) ? "▾" : "▸") : "";
  // inline axis tags turn the count into the answer (deduped — the
  // detail aside keeps the full list)
  const tags = own
    ? `<i class="tags">${[...new Set(n.axes)].map((a) => esc(axisName(a))).join(" · ")}</i>`
    : "";
  // the chip carries its own relief (a number on the heat color, ink
  // flipped dark on the hot end) — never color-alone
  const chip = own
    ? `<span class="fchip" style="background:${heat(own)};color:${own >= 3 ? "#12151a" : "#f4efe6"}">${own}</span>`
    : `<span class="fchip"></span>`;
  // data-id = what a click SELECTS (the node whose detail shows);
  // data-row = what a twirl EXPANDS (the chain head, which also sets
  // the indent). Two ids because a folded row stands for a chain.
  return (
    `<div class="trow${tail === selId ? " sel" : ""}" data-id="${tail}" data-row="${head}"` +
    ` style="--d:${n.depth - (label.split("/").length - 1)};border-left-color:${railColor(tail, head, ctx)}"` +
    ` title="${esc(n.name || ".")}  ${tr("findings")} ${own} · Σ ${ctx.roll[tail]}">` +
    `<span class="tw" data-tw>${twirl}</span>` +
    `<span class="tname">${esc(label)}${tags}</span>${chip}` +
    `<span class="tsub">${ctx.roll[tail] || ""}</span>` +
    `<span class="tmeta">${ctx.filesBelow[tail]}</span></div>`
  );
}

function selectRow(id) {
  selId = id;
  $("structree").querySelectorAll(".trow.sel").forEach((r) => r.classList.remove("sel"));
  $("structree").querySelector(`.trow[data-id="${id}"]`)?.classList.add("sel");
  showDetail(id);
}

// One delegated listener: the twirl toggles, anywhere else selects.
$("structree").addEventListener("click", (e) => {
  const rowEl = e.target.closest(".trow");
  if (!rowEl || rowEl.classList.contains("thead")) return;
  const row = Number(rowEl.dataset.row);
  const id = Number(rowEl.dataset.id);
  if (e.target.closest("[data-tw]") && children[id].length) {
    if (!expanded.delete(row)) expanded.add(row);
    drawTree();
  } else {
    selectRow(id);
  }
});

$("vswitch").addEventListener("click", (e) => {
  const b = e.target.closest("button");
  if (!b) return;
  treeView = b.dataset.v === "tree";
  localStorage.setItem("ce-structure-view", treeView ? "tree" : "map");
  redrawStructure();
});
