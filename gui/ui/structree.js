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

function drawTree() {
  if (lastTree !== report.tree) {
    lastTree = report.tree;
    // three levels open by default; hot rails flag the folded rest
    expanded = new Set(report.tree.flatMap((n, i) => (n.depth < 3 ? [i] : [])));
  }
  const ctx = {
    roll: rollup((n) => n.axes.length),
    filesBelow: rollup((n) => n.files),
  };
  ctx.mx = Math.max(1, ...ctx.roll.slice(1));
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

// One row per BRANCHING node: unary chains (core/app/CE …) fold into
// a single row exactly like the treemap's strips, so both lenses
// name the same places the same way.
function walkRows(id, ctx, out) {
  const path = [id];
  while (children[path[path.length - 1]].length === 1) {
    path.push(children[path[path.length - 1]][0]);
  }
  const tail = path[path.length - 1];
  const label = path
    .map((i) => report.tree[i].name.split("/").pop() || ".")
    .join("/");
  out.push(rowHtml(tail, label, ctx));
  if (expanded.has(tail)) {
    for (const k of children[tail]) walkRows(k, ctx, out);
  }
}

// Rail color: log-ranked against the report's own maximum, so the
// hottest branch is unique and the low end stays separable (a fixed
// 0..4 clamp painted most rows the same top stop). The root is
// exempt — its rollup is the grand total the summary strip already
// shows, and it would always be the hottest row on screen.
function railColor(tail, ctx) {
  const r = ctx.roll[tail];
  if (tail === 0) return "transparent";
  if (!r) return LEGEND_STOPS[0];
  return LEGEND_STOPS[1 + Math.floor((3 * Math.log1p(r)) / Math.log1p(ctx.mx))];
}

function rowHtml(tail, label, ctx) {
  const n = report.tree[tail];
  const own = n.axes.length;
  const twirl = children[tail].length ? (expanded.has(tail) ? "▾" : "▸") : "";
  // inline axis tags turn the count into the answer (deduped — the
  // detail aside keeps the full list)
  const tags = own
    ? `<i class="tags">${[...new Set(n.axes)].map((a) => esc(tr("axisNames")[a])).join(" · ")}</i>`
    : "";
  // the chip carries its own relief (a number on the heat color, ink
  // flipped dark on the hot end) — never color-alone
  const chip = own
    ? `<span class="fchip" style="background:${heat(own)};color:${own >= 3 ? "#12151a" : "#f4efe6"}">${own}</span>`
    : `<span class="fchip"></span>`;
  return (
    `<div class="trow${tail === selId ? " sel" : ""}" data-id="${tail}"` +
    ` style="--d:${n.depth - (label.split("/").length - 1)};border-left-color:${railColor(tail, ctx)}"` +
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
  const id = Number(rowEl.dataset.id);
  if (e.target.closest("[data-tw]") && children[id].length) {
    if (!expanded.delete(id)) expanded.add(id);
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
