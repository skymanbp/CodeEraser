// CodeEraser GUI glue (M6 S4a structure screen + M7-P4 tab shell +
// M8-G3a i18n). This file RENDERS the report document — every number
// it shows was judged in the core and measured in the codeeraser
// crate; nothing is derived here beyond layout geometry. Labels come
// from i18n.js (tr / axis names); `invoke`, `$`, `esc`, `row`,
// `setStatus`, `posInt` and `redrawers` are the shared globals the
// sibling screens use.
"use strict";

const invoke = window.__TAURI__.core.invoke;
const $ = (id) => document.getElementById(id);

// Per-screen redraw hooks: switching back to a tab re-renders its
// SVG, because a chart drawn (or resized) while its view was hidden
// measured a 0×0 rect and holds no usable geometry.
const redrawers = {};

let report = null;
let children = [];
let selId = 0;

function tabs() {
  const all = document.querySelectorAll("#tabs .tab");
  all.forEach((b) =>
    b.addEventListener("click", () => {
      all.forEach((x) => x.classList.toggle("on", x === b));
      document.querySelectorAll(".view").forEach((v) => {
        v.hidden = v.id !== "view-" + b.dataset.tab;
      });
      redrawers[b.dataset.tab]?.();
    }));
}

// One status line for all three screens; long errors ellipsize in
// CSS, so the full text rides on the tooltip.
function setStatus(text, isErr) {
  const s = $("status");
  s.className = isErr ? "err" : "";
  s.textContent = text;
  s.title = isErr ? text : "";
}

// Number inputs guard here, not just in the HTML min attribute —
// typed garbage or a negative would otherwise reach the backend as a
// deserialization error. Invalid → fallback (null = axis unjudged).
function posInt(value, min, fallback) {
  const n = Math.floor(Number(value));
  return Number.isFinite(n) && n >= min ? n : fallback;
}

async function boot() {
  applyStaticI18n();
  $("lang").addEventListener("click", toggleLang);
  i18nRefreshers.push(() => report && render());
  $("root").value = await invoke("default_root").catch(() => "");
  tabs();
  $("scan").addEventListener("click", scan);
  $("root").addEventListener("keydown", (e) => e.key === "Enter" && scan());
  // redrawStructure (structree.js) routes to the active lens
  window.addEventListener("resize", () => report && redrawStructure());
  redrawers.structure = () => report && redrawStructure();
}

async function scan() {
  const days = $("days").value ? posInt($("days").value, 1, null) : null;
  $("scan").disabled = true;
  setStatus(tr("judging"), false);
  try {
    report = await invoke("structure_report", {
      root: $("root").value,
      deep: $("deep").checked,
      days,
    });
    if (selId >= report.tree.length) selId = 0;
    render();
    setStatus(report.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("scan").disabled = false;
  }
}

// Chips, not joined strings: HTML collapses whitespace runs, so a
// two-space separator renders identical to the one-space inside a
// pair and the grouping is lost. Zero-count axes recede via .zero.
function render() {
  $("empty-structure").hidden = true;
  $("summary").hidden = false;
  $("score").textContent = report.score;
  $("scale").textContent = "/ " + report.scoreScale;
  // the fill is score/scale as width — chart geometry over two
  // report facts, same stance as the trend chart's y position
  $("scorefill").style.width = `${(100 * report.score) / report.scoreScale}%`;
  $("entropy").innerHTML = report.entropy
    .map(([k, v]) => `<span>${esc(tr("entropyNames")[k] ?? String(k))} <b>${v}‰</b></span>`)
    .join("");
  $("axes").innerHTML = report.axes
    .map(([c, p]) => `<span${p === 0 ? ' class="zero"' : ""}>${esc(tr("axisNames")[c])} <b>${p}</b></span>`)
    .join("");
  $("divergence").textContent =
    report.declaredDirs > 0
      ? report.divergence === null
        ? tr("divergenceOutside")
        : `χ² ${report.divergence}‰`
      : "";
  // legend swatches come from LEGEND_STOPS so the ramp has one source
  $("legend").hidden = false;
  $("legend").innerHTML =
    LEGEND_STOPS.map((c) => `<i style="background:${c}"></i>`).join("") +
    `<span>${tr("legend")}</span>`;
  children = report.tree.map(() => []);
  report.tree.forEach((n, i) => {
    if (i > 0) children[n.parent].push(i);
  });
  redrawStructure();
  showDetail(selId);
}

// Subtree weight = every file under the node (+1 keeps empty dirs
// visible); computed once per render, leaf-to-root over the dense
// parent-before-child order the report guarantees.
function weights() {
  const w = report.tree.map((n) => n.files + 1);
  for (let i = report.tree.length - 1; i > 0; i--) {
    w[report.tree[i].parent] += w[i];
  }
  return w;
}

function drawTreemap() {
  const svg = $("treemap");
  const { width, height } = svg.getBoundingClientRect();
  if (!width || !height) return; // hidden view measures 0×0 — keep the old drawing
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.textContent = "";
  const GUT = 14; // the app-wide left/right rail
  layout(0, GUT, GUT, width - 2 * GUT, height - 2 * GUT, weights());
}

// Squarified treemap (Bruls, Huizing & van Wijk 2000): each node
// claims a name strip, its children fill the rest largest-first in
// rows placed along the short side, each row frozen when adding the
// next child would worsen its worst aspect ratio.
function layout(id, x, y, wd, ht, w) {
  if (wd < 3 || ht < 3) return;
  // Unary chains (core/app/CE, gui/src-tauri/src …) collapse to ONE
  // strip: a path of single children burns a 16px band per level and
  // biases the area encoding against deep subtrees. One branching
  // level, one strip; the joined path is the label.
  const path = [id];
  while (children[path[path.length - 1]].length === 1) {
    path.push(children[path[path.length - 1]][0]);
  }
  const tail = path[path.length - 1];
  drawRect(tail, x, y, wd, ht, path.map((i) => report.tree[i].name.split("/").pop() || ".").join("/"));
  const kids = children[tail];
  if (!kids.length) return;
  const strip = ht > 30 ? 16 : 0;
  const ix = x + 2;
  const iy = y + 2 + strip;
  const iw = wd - 4;
  const ih = ht - 4 - strip;
  if (iw < 3 || ih < 3) return;
  const total = kids.reduce((s, k) => s + w[k], 0) || 1;
  const scale = (iw * ih) / total;
  const items = [...kids].sort((a, b) => w[b] - w[a]).map((k) => ({ k, area: w[k] * scale }));
  squarify(items, ix, iy, iw, ih, w);
}

// The worst (largest) aspect ratio a row would have at this thickness.
function worst(row, sum, side) {
  const thick = sum / side;
  let m = 0;
  for (const it of row) {
    const len = it.area / thick;
    m = Math.max(m, len / thick, thick / len);
  }
  return m;
}

function squarify(items, x, y, wd, ht, w) {
  let i = 0;
  while (i < items.length && wd > 0 && ht > 0) {
    const side = Math.min(wd, ht);
    const row = [items[i]];
    let sum = items[i].area;
    let j = i + 1;
    while (j < items.length && worst([...row, items[j]], sum + items[j].area, side) <= worst(row, sum, side)) {
      row.push(items[j]);
      sum += items[j].area;
      j += 1;
    }
    const thick = sum / side;
    let off = 0;
    for (const it of row) {
      const len = it.area / thick;
      if (wd <= ht) layout(it.k, x + off, y, len, thick, w);
      else layout(it.k, x, y + off, thick, len, w);
      off += len;
    }
    if (wd <= ht) {
      y += thick;
      ht -= thick;
    } else {
      x += thick;
      wd -= thick;
    }
    i = j;
  }
}

function drawRect(id, x, y, wd, ht, label) {
  const n = report.tree[id];
  const ns = "http://www.w3.org/2000/svg";
  const r = document.createElementNS(ns, "rect");
  r.setAttribute("x", x);
  r.setAttribute("y", y);
  r.setAttribute("width", Math.max(wd, 0));
  r.setAttribute("height", Math.max(ht, 0));
  r.setAttribute("rx", 2);
  r.setAttribute("fill", heat(n.axes.length));
  // the root's ring would frame the whole canvas as one giant
  // "selected" box — the detail pane still shows it, ring skipped
  if (id === selId && id !== 0) r.classList.add("sel");
  r.addEventListener("click", (e) => {
    e.stopPropagation();
    selectNode(id, r);
  });
  const title = document.createElementNS(ns, "title");
  title.textContent = `${n.name}  ${tr("files")} ${n.files}  ${tr("findings")} ${n.axes.length}`;
  r.appendChild(title);
  $("treemap").appendChild(r);
  if (wd > 44 && ht > 26) {
    const t = document.createElementNS(ns, "text");
    t.setAttribute("x", x + 5);
    t.setAttribute("y", y + 13);
    // middle-truncate: siblings share prefixes (eval_graph_…, eval_l2_…),
    // so tail-cutting deletes exactly the discriminating half
    const fit = Math.floor((wd - 10) / 6.6);
    if (label.length > fit && fit >= 3) {
      const head = Math.ceil((fit - 1) / 2);
      t.textContent = label.slice(0, head) + "…" + label.slice(-(fit - 1 - head) || -1);
    } else {
      t.textContent = label.length > fit ? label.slice(0, 1) + "…" : label;
    }
    $("treemap").appendChild(t);
  }
}

// The alarm ramp: 0 findings = calm panel green-grey; the four hot
// stops sit on even lightness steps so adjacent counts stay
// separable (capped at 4 — beyond that it is simply the hot end).
// LEGEND_STOPS mirrors it for the summary legend.
const LEGEND_STOPS = ["#222932", "#61332a", "#93402f", "#c2503d", "#e8705a"];
function heat(findings) {
  return LEGEND_STOPS[Math.min(findings, LEGEND_STOPS.length - 1)];
}

function selectNode(id, rect) {
  selId = id;
  $("treemap").querySelectorAll("rect.sel").forEach((r) => r.classList.remove("sel"));
  if (id !== 0) rect.classList.add("sel");
  showDetail(id);
}

function showDetail(id) {
  const n = report.tree[id];
  const rows = [
    `<h2>${esc(n.name)}</h2>`,
    row(tr("depth"), n.depth),
    row(tr("subdirs"), n.subdirs),
    row(tr("files"), n.files),
    row(tr("findings"), n.axes.map((a) => tr("axisNames")[a]).join(", ") || tr("none")),
  ];
  for (const d of report.deviations) {
    if (d.dir === n.name) {
      rows.push(row(tr("deviation"), d.kind === 0 ? tr("undeclared") : tr("declaredEmpty")));
    }
  }
  $("detail").innerHTML = rows.join("");
}

const row = (k, v) => `<div class="row">${k}: <b>${esc(String(v))}</b></div>`;
const esc = (s) =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

boot();
