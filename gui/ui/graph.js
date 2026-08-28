// CodeEraser GUI — the graph screen (batch 9 P18, user-ruled): the
// ce.graph-canvas document as a drawn map. File tier only; the
// layout is deterministic (no random seed — the same document draws
// the same picture); dead files and cycle members ride the alarm ramp;
// teal is selection only; magnitude is geometry (node radius =
// degree), never a printed number. Rendering only: every verdict on
// this screen came from the one core judgment the CLI also prints.
//
// The symbol-level drill-down (plan v2.17 L round piece (7)) reads a
// SECOND judgment's document beside the canvas one: the
// ce.deadcode-report's `unmentioned` advisory rows, grouped here by
// file path — a rendering join on a string the two documents share,
// never a verdict, and best-effort by path since the two are separate
// runs. The advisory has one home (that report); this screen only
// shows it beside the file it names.
"use strict";

let graphDoc = null;
let advDoc = null;
// why the advisory road failed while the map drew — its third state
let advWhy = "";
let advByPath = new Map();
let gpos = null, gsel = -1;

(function bootGraph() {
  i18nRefreshers.push(() => graphDoc && (renderGraphAside(gsel), drawGraph()));
  $("graph-load").addEventListener("click", loadGraph);
  const c = $("graphcanvas");
  c.addEventListener("mousemove", graphHover);
  c.addEventListener("click", graphClick);
  redrawers.graph = () => graphDoc && drawGraph();
})();

async function loadGraph() {
  $("graph-load").disabled = true;
  setStatus(tr("judging"));
  try {
    // The canvas road is authoritative: the map commits on it alone,
    // and both documents commit together so a failure never leaves
    // the map on a new run beside an advisory from the old one. The
    // advisory road settles on its own — a pre-6.2.0 core refuses it
    // by name while the canvas (Advisory::No) still draws — and its
    // failure is a third state the aside names, never a silent zero.
    const root = $("root").value;
    const [g, d] = await Promise.allSettled([
      invoke("graphcanvas_report", { root }),
      invoke("deadcode_report", { root }),
    ]);
    if (g.status === "rejected") throw g.reason;
    graphDoc = g.value;
    advDoc = d.status === "fulfilled" ? d.value : null;
    advWhy = d.status === "fulfilled" ? "" : String(d.reason);
    advByPath = new Map();
    for (const a of advDoc?.unmentioned ?? []) {
      if (!advByPath.has(a.name)) advByPath.set(a.name, []);
      advByPath.get(a.name).push(a);
    }
    $("empty-graph").hidden = true;
    gsel = -1;
    layoutGraph();
    drawGraph();
    renderGraphAside(-1);
    setStatus(graphDoc.schema);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("graph-load").disabled = false;
  }
}

function layoutGraph() {
  const n = graphDoc.files.length;
  gpos = Array.from({ length: n }, (_, i) => {
    const a = i * 2.399963, r = Math.sqrt(i + 0.5);
    return { x: r * Math.cos(a), y: r * Math.sin(a) };
  });
  relaxGraph(Math.max(40, Math.min(250, Math.floor(3e7 / Math.max(1, n * n)))));
}

function relaxGraph(iters) {
  let cap = 0.5;
  for (let it = 0; it < iters; it++) {
    const d = gpos.map(() => ({ x: 0, y: 0 }));
    for (let i = 0; i < gpos.length; i++) {
      for (let j = i + 1; j < gpos.length; j++) {
        const dx = gpos[i].x - gpos[j].x, dy = gpos[i].y - gpos[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy + 0.01);
        const f = Math.min(cap, 1.2 / (dx * dx + dy * dy + 0.01));
        d[i].x += f * dx / dist; d[i].y += f * dy / dist;
        d[j].x -= f * dx / dist; d[j].y -= f * dy / dist;
      }
    }
    graphDoc.edges.forEach(([a, b]) => {
      const dx = gpos[b].x - gpos[a].x, dy = gpos[b].y - gpos[a].y;
      const dist = Math.sqrt(dx * dx + dy * dy + 0.01), f = (dist - 1) * 0.05;
      d[a].x += f * dx / dist; d[a].y += f * dy / dist;
      d[b].x -= f * dx / dist; d[b].y -= f * dy / dist;
    });
    gpos.forEach((p, i) => {
      const m = Math.sqrt(d[i].x * d[i].x + d[i].y * d[i].y) || 1;
      const s = Math.min(cap, m) / m;
      p.x += d[i].x * s; p.y += d[i].y * s;
    });
    cap *= 0.97;
  }
}

function fitGraph(w, h) {
  if (!gpos || !gpos.length) return null;
  const xs = gpos.map((p) => p.x), ys = gpos.map((p) => p.y);
  const minX = Math.min(...xs), maxX = Math.max(...xs), minY = Math.min(...ys), maxY = Math.max(...ys);
  const sx = (w - 48) / Math.max(0.001, maxX - minX), sy = (h - 48) / Math.max(0.001, maxY - minY);
  const scale = Math.min(sx, sy), ox = (w - scale * (maxX - minX)) / 2;
  const oy = (h - scale * (maxY - minY)) / 2;
  return (p) => ({ x: ox + (p.x - minX) * scale, y: oy + (p.y - minY) * scale });
}

function graphColors() {
  const s = getComputedStyle(document.documentElement);
  return ["--line", "--panel-2", "--focus", "--ink-low"].reduce((o, k) => {
    o[k] = s.getPropertyValue(k).trim(); return o;
  }, {});
}

function drawGraph() {
  const c = $("graphcanvas"), w = c.clientWidth, h = c.clientHeight, dpr = devicePixelRatio || 1;
  if (!w || !h || !gpos || !gpos.length) return;
  c.width = w * dpr; c.height = h * dpr;
  const ctx = c.getContext("2d"), map = fitGraph(w, h), colors = graphColors();
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  drawEdges(ctx, map, colors);
  drawNodes(ctx, map, colors);
}

function edgeIsSelected(a, b) { return gsel >= 0 && (a === gsel || b === gsel); }

function drawEdges(ctx, map, colors) {
  ctx.strokeStyle = colors["--line"]; ctx.globalAlpha = 0.45; ctx.lineWidth = 1;
  graphDoc.edges.forEach(([a, b]) => {
    const p = map(gpos[a]), q = map(gpos[b]);
    ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(q.x, q.y); ctx.stroke();
  });
  if (gsel < 0) { ctx.globalAlpha = 1; return; }
  ctx.strokeStyle = colors["--ink-low"]; ctx.globalAlpha = 0.9;
  graphDoc.edges.forEach(([a, b]) => {
    if (!edgeIsSelected(a, b)) return;
    const p = map(gpos[a]), q = map(gpos[b]);
    ctx.beginPath(); ctx.moveTo(p.x, p.y); ctx.lineTo(q.x, q.y); ctx.stroke();
  });
  ctx.globalAlpha = 1;
}

function nodeRadius(file) {
  const p = file.pos;
  return p == null ? 3 : Math.min(12, 3 + 2 * Math.sqrt(p[0] + p[1]));
}

function drawNodes(ctx, map, colors) {
  graphDoc.files.forEach((file, i) => {
    // `cycle` is the core's membership bit off the document (0.3.0);
    // this screen never re-derives the SCC floor from pos[3]
    const p = map(gpos[i]), dead = file.verdict != null, cycle = file.cycle === true;
    const radius = nodeRadius(file), fill = dead ? heat(4) : cycle ? heat(1) : colors["--panel-2"];
    const ring = dead ? heat(3) : cycle ? heat(2) : colors["--line"];
    ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, Math.PI * 2); ctx.fillStyle = fill; ctx.fill();
    ctx.strokeStyle = ring; ctx.lineWidth = 1.5; ctx.stroke();
    if (i === gsel) {
      ctx.beginPath(); ctx.arc(p.x, p.y, radius + 3, 0, Math.PI * 2);
      ctx.strokeStyle = colors["--focus"]; ctx.lineWidth = 2; ctx.stroke();
    }
  });
}

function graphHit(evt) {
  if (!graphDoc || !gpos) return -1;
  const c = $("graphcanvas"), r = c.getBoundingClientRect(), map = fitGraph(c.clientWidth, c.clientHeight);
  const x = evt.clientX - r.left, y = evt.clientY - r.top;
  let best = -1, dist = Infinity;
  graphDoc.files.forEach((file, i) => {
    const p = map(gpos[i]), d = Math.hypot(x - p.x, y - p.y);
    if (d <= Math.max(10, nodeRadius(file) + 2) && d < dist) { best = i; dist = d; }
  });
  return best;
}

function graphHover(evt) {
  const i = graphHit(evt), c = $("graphcanvas");
  if (i < 0) c.title = "";
  else {
    const path = graphDoc.files[i].path, n = (advByPath.get(path) ?? []).length;
    c.title = n ? `${path} · ${tr("advisoryHover", n)}` : path;
  }
  c.classList.toggle("pick", i >= 0);
}

function graphClick(evt) {
  gsel = graphHit(evt);
  drawGraph();
  renderGraphAside(gsel);
}

// The advisory's three states the rows cannot show for themselves:
// the road itself failed (no document — the map still drew), the core
// dropped the table (nothing judged at symbol level), or the producer
// cut the candidate set (the rows are a prefix). The last two come off
// the report document's own flags, as the CLI prints them.
function advisoryNotices() {
  if (!advDoc) return `<div class="notice"><b>${esc(tr("advisoryUnavailable"))}</b><small>${esc(advWhy)}</small></div>`;
  let html = "";
  if (advDoc.unmentioned_dropped) html += `<div class="notice"><b>${esc(tr("advisoryDropped"))}</b></div>`;
  if (advDoc.unmentioned_cut) html += `<div class="notice"><b>${esc(tr("advisoryCut"))}</b></div>`;
  return html;
}

// The whole-tree census of the advisory, by the core's code: counting
// is not judging, every code came off the wire.
function advisoryCensus() {
  if (!advDoc) return advisoryNotices();
  const rows = advDoc.unmentioned ?? [];
  const words = tr("advisoryWords");
  const by = new Map();
  for (const a of rows) by.set(a.code, (by.get(a.code) ?? 0) + 1);
  let html = `<div class="row zero">${esc(tr("advisoryHead", rows.length, advByPath.size))}</div>`;
  for (const [code, n] of by) html += row(words[code] ?? code, n);
  return html + advisoryNotices();
}

function renderGraphAside(i) {
  const d = graphDoc, el = $("graph-detail");
  if (i < 0 || !d) {
    const c = d ? d.counts : { files: 0, edges: 0, dead: 0, cycles: 0 };
    let html = `<h2>${esc(tr("tabGraph"))}</h2>` +
      `<div class="row">${esc(tr("graphCounts", c.files, c.edges, c.dead, c.cycles))}</div>`;
    if (d && d.unresolvedSites > 0) html += `<div class="row zero">${esc(tr("graphUnresolved", d.unresolvedSites))}</div>`;
    if (d && d.degraded) html += `<div class="notice"><b>${esc(tr("degradedRun"))}</b><small>${esc(String(d.degraded))}</small></div>`;
    if (d) html += advisoryCensus();
    el.innerHTML = html;
    return;
  }
  const f = d.files[i], p = f.pos;
  let html = `<h2>${esc(String(f.path))}</h2>`;
  if (f.verdict != null) {
    html += `<div class="row"><b>${esc(String(f.verdict))}</b></div><div class="row zero">${esc(String(f.why ?? ""))}</div>`;
    // The trust column (2.32.0) the console prints beside the verdict
    // and this pane dropped. The same number decides whether `ce
    // erase` may act on the row, so a verdict shown without it is a
    // stronger claim than the core made.
    if (f.conf != null) {
      const word = tr("trustNames")[f.conf] ?? String(f.conf);
      html += `<div class="row ${f.conf === 0 ? "bad" : "zero"}"><span>${esc(tr("trust"))}</span>${esc(word)}</div>`;
    }
  } else html += `<div class="row">${esc(tr("graphAlive"))}</div>`;
  if (p != null) {
    html += `<div class="row">${esc(tr("graphInOut", p[0], p[1]))}</div>`;
    if (f.cycle === true) html += `<div class="row">${esc(tr("graphCycleOf", p[3]))}</div>`;
  }
  el.innerHTML = html + advisoryOf(f.path);
}

// The file's own unmentioned declarations (the symbol-level
// drill-down): line, name and the core's code word, the code's
// reading on the tooltip. A file with none says so in one quiet row
// rather than showing nothing — "no advisory", "not loaded" and "the
// road failed" must not look alike.
function advisoryOf(path) {
  if (!advDoc) return advisoryNotices();
  const rows = advByPath.get(path) ?? [];
  const words = tr("advisoryWords");
  let html = `<div class="row zero">${esc(tr("advisoryHead", rows.length, rows.length ? 1 : 0))}</div>`;
  for (const a of rows) {
    html += `<div class="row" title="${esc(String(a.why))}"><span class="dir">${esc(String(a.line))}</span> ` +
      `<b>${esc(String(a.symbol))}</b> <span class="dir">${esc(words[a.code] ?? String(a.code))}</span></div>`;
  }
  return html + advisoryNotices();
}
