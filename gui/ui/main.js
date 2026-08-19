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
  window.addEventListener("resize", () => report && drawTreemap());
  redrawers.structure = () => report && drawTreemap();
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

function render() {
  $("empty-structure").hidden = true;
  $("summary").hidden = false;
  $("score").textContent = report.score;
  $("scale").textContent = "/ " + report.scoreScale;
  $("entropy").textContent =
    tr("entropy") + " " + report.entropy.map(([k, v]) => `${k}:${v}‰`).join("  ");
  $("axes").textContent = report.axes
    .map(([c, p]) => `${tr("axisNames")[c]} ${p}`)
    .join("  ");
  $("divergence").textContent =
    report.declaredDirs > 0
      ? report.divergence === null
        ? tr("divergenceOutside")
        : `χ² ${report.divergence}‰`
      : "";
  children = report.tree.map(() => []);
  report.tree.forEach((n, i) => {
    if (i > 0) children[n.parent].push(i);
  });
  drawTreemap();
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
  const w = weights();
  layout(0, 2, 2, width - 4, height - 4, true, w);
}

// Slice-and-dice with alternating orientation: the node claims a
// name strip, its children split the rest by subtree weight.
function layout(id, x, y, wd, ht, horiz, w) {
  if (wd < 3 || ht < 3) return;
  drawRect(id, x, y, wd, ht);
  const kids = children[id];
  if (!kids.length) return;
  const strip = ht > 30 ? 16 : 0;
  let cx = x + 2;
  let cy = y + 2 + strip;
  const iw = wd - 4;
  const ih = ht - 4 - strip;
  if (iw < 3 || ih < 3) return;
  const total = kids.reduce((s, k) => s + w[k], 0) || 1;
  for (const k of kids) {
    const frac = w[k] / total;
    if (horiz) {
      layout(k, cx, cy, iw * frac, ih, !horiz, w);
      cx += iw * frac;
    } else {
      layout(k, cx, cy, iw, ih * frac, !horiz, w);
      cy += ih * frac;
    }
  }
}

function drawRect(id, x, y, wd, ht) {
  const n = report.tree[id];
  const ns = "http://www.w3.org/2000/svg";
  const r = document.createElementNS(ns, "rect");
  r.setAttribute("x", x);
  r.setAttribute("y", y);
  r.setAttribute("width", Math.max(wd, 0));
  r.setAttribute("height", Math.max(ht, 0));
  r.setAttribute("fill", heat(n.axes.length));
  if (id === selId) r.setAttribute("class", "sel");
  r.addEventListener("click", (e) => {
    e.stopPropagation();
    selectNode(id);
  });
  const title = document.createElementNS(ns, "title");
  title.textContent = `${n.name}  ${tr("files")} ${n.files}  ${tr("findings")} ${n.axes.length}`;
  r.appendChild(title);
  $("treemap").appendChild(r);
  if (wd > 60 && ht > 26) {
    const t = document.createElementNS(ns, "text");
    t.setAttribute("x", x + 5);
    t.setAttribute("y", y + 13);
    // ~6.6px per glyph at 11px mono; clip so labels never spill
    // across a sibling rectangle.
    const label = n.name.split("/").pop() || ".";
    const fit = Math.floor((wd - 10) / 6.6);
    t.textContent = label.length > fit ? label.slice(0, Math.max(fit - 1, 1)) + "…" : label;
    $("treemap").appendChild(t);
  }
}

// The alarm ramp: 0 findings = calm panel green-grey; each finding
// steps toward the hot end (capped at 4 — beyond that it is simply
// red).
function heat(findings) {
  if (findings === 0) return "#222932";
  const stops = ["#4a3a33", "#6d4136", "#8f4439", "#b0483f"];
  return stops[Math.min(findings, stops.length) - 1];
}

function selectNode(id) {
  selId = id;
  $("treemap").querySelectorAll("rect").forEach((r) => r.removeAttribute("class"));
  const rects = $("treemap").querySelectorAll("rect");
  // rects append in layout() DFS order, not tree order — re-render
  // detail and let drawTreemap restore the highlight next redraw.
  showDetail(id);
  rects.forEach((r) => {
    if (r.querySelector("title")?.textContent.startsWith(report.tree[id].name + "  ")) {
      r.setAttribute("class", "sel");
    }
  });
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
