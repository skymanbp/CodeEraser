// CodeEraser GUI glue (M6 S4a structure screen + M7-P4 tab shell).
// This file RENDERS the report document — every number it shows was
// judged in the core and measured in the codeeraser crate; nothing
// is derived here beyond layout geometry. Axis labels mirror the
// design booklet §3 table. The trend / candidates screens live in
// their own files (the repo's own 300-line gate applies to JS too);
// `invoke`, `$`, `esc` and `row` here are the shared globals.
"use strict";

const invoke = window.__TAURI__.core.invoke;
const $ = (id) => document.getElementById(id);
const AXIS = [
  "geometry", "naming", "mixing", "misplaced",
  "docs", "stale-docs", "redundancy",
];

let report = null;
let children = [];

// Tab switching is pure visibility — each screen keeps its own state.
function tabs() {
  const all = document.querySelectorAll("#tabs .tab");
  all.forEach((b) =>
    b.addEventListener("click", () => {
      all.forEach((x) => x.classList.toggle("on", x === b));
      document.querySelectorAll(".view").forEach((v) => {
        v.hidden = v.id !== "view-" + b.dataset.tab;
      });
    }));
}

async function boot() {
  $("root").value = await invoke("default_root").catch(() => "");
  tabs();
  $("scan").addEventListener("click", scan);
  $("root").addEventListener("keydown", (e) => e.key === "Enter" && scan());
  window.addEventListener("resize", () => report && drawTreemap());
}

async function scan() {
  const days = $("days").value ? Number($("days").value) : null;
  $("scan").disabled = true;
  $("status").className = "";
  $("status").textContent = "judging…";
  try {
    report = await invoke("structure_report", {
      root: $("root").value,
      deep: $("deep").checked,
      days,
    });
    render();
    $("status").textContent = report.schema;
  } catch (e) {
    $("status").className = "err";
    $("status").textContent = String(e);
  } finally {
    $("scan").disabled = false;
  }
}

function render() {
  $("summary").hidden = false;
  $("score").textContent = report.score;
  $("scale").textContent = "/ " + report.scoreScale;
  $("entropy").textContent =
    "entropy " + report.entropy.map(([k, v]) => `${k}:${v}‰`).join("  ");
  $("axes").textContent = report.axes
    .map(([c, p]) => `${AXIS[c]} ${p}`)
    .join("  ");
  $("divergence").textContent =
    report.declaredDirs > 0
      ? report.divergence === null
        ? "divergence: mass outside declared dirs"
        : `divergence ${report.divergence}‰`
      : "";
  children = report.tree.map(() => []);
  report.tree.forEach((n, i) => {
    if (i > 0) children[n.parent].push(i);
  });
  drawTreemap();
  showDetail(0);
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
  r.addEventListener("click", (e) => {
    e.stopPropagation();
    showDetail(id);
  });
  const title = document.createElementNS(ns, "title");
  title.textContent = `${n.name}  files ${n.files}  findings ${n.axes.length}`;
  r.appendChild(title);
  $("treemap").appendChild(r);
  if (wd > 60 && ht > 26) {
    const t = document.createElementNS(ns, "text");
    t.setAttribute("x", x + 5);
    t.setAttribute("y", y + 13);
    t.textContent = n.name.split("/").pop() || ".";
    $("treemap").appendChild(t);
  }
}

// The alarm ramp: 0 findings = calm panel green-grey; each finding
// steps toward the hot end (capped at 4 — beyond that it is simply
// red).
function heat(findings) {
  if (findings === 0) return "#232b31";
  const stops = ["#4a3a33", "#6d4136", "#8f4439", "#b0483f"];
  return stops[Math.min(findings, stops.length) - 1];
}

function showDetail(id) {
  const n = report.tree[id];
  const rows = [
    `<h2>${esc(n.name)}</h2>`,
    row("depth", n.depth),
    row("subdirs", n.subdirs),
    row("files", n.files),
    row("findings", n.axes.map((a) => AXIS[a]).join(", ") || "none"),
  ];
  for (const d of report.deviations) {
    if (d.dir === n.name) {
      rows.push(row("deviation", d.kind === 0 ? "undeclared territory" : "declared but empty"));
    }
  }
  $("detail").innerHTML = rows.join("");
}

const row = (k, v) => `<div class="row">${k}: <b>${esc(String(v))}</b></div>`;
const esc = (s) =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

boot();
