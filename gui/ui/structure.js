// CodeEraser GUI — the structure screen (split from main.js at the
// repo's own soft line, batch 4). RENDERS the report document —
// every number it shows was judged in the core and measured in the
// codeeraser crate; nothing is derived here beyond layout geometry.
// Shares `report`/`children`/`selId` with structree.js (the tree
// lens) and consumes the shell globals from app.js.
"use strict";

let report = null;
let children = [];
let selId = 0;

function bootStructure() {
  i18nRefreshers.push(() => report && render());
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
      split: $("split").checked,
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
    .map(([c, p]) => `<span${p === 0 ? ' class="zero"' : ""}>${esc(axisName(c))} <b>${p}</b></span>`)
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
  renderSplit();
  redrawStructure();
  showDetail(selId);
}

// The split-ROI advisory rows (v0.7, plan v2.7 ③): the SAME priced
// verdicts `ce structure --split-candidates` prints — a candidate
// names its seam line and unit, an exemption its machine-written
// why in numbers. Rendering only; nothing is priced here.
function renderSplit() {
  const p = $("splitpanel");
  p.hidden = !report.split;
  if (!report.split) return;
  const cand = report.splitCandidates.map(
    (c) =>
      `<div class="row">✂ <b>${esc(c.path)}</b> — ${esc(tr("seamAfter"))} L${c.afterLine}` +
      ` (${esc(c.unit)}) · +${c.benefitMilli}‰ / −${c.costMilli}‰</div>`,
  );
  const ex = report.sizeExempt.map(
    (e) =>
      `<div class="row zero">${esc(e.path)} — ${esc(tr("cohesive"))}` +
      ` (${e.benefitMilli}‰ / ${e.costMilli}‰)</div>`,
  );
  p.innerHTML =
    `<b>${esc(tr("splitTitle"))}</b>` + (cand.join("") + ex.join("") || esc(tr("none")));
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
  // Empty unary chains (core/app/CE, gui/src-tauri/src …) collapse to
  // ONE strip: a path of single children burns a 16px band per level
  // and biases the area encoding against deep subtrees. One branching
  // level, one strip; the joined path is the label. foldChain
  // (structree.js) is the SHARED rule, so the two lenses cannot
  // disagree about which directories they name — and it never folds
  // away a node that carries findings of its own.
  const path = foldChain(id);
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
    row(tr("findings"), n.axes.map(axisName).join(", ") || tr("none")),
  ];
  for (const d of report.deviations) {
    if (d.dir === n.name) {
      rows.push(row(tr("deviation"), d.kind === 0 ? tr("undeclared") : tr("declaredEmpty")));
    }
  }
  $("detail").innerHTML = rows.join("");
}
