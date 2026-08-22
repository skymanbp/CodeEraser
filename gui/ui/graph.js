// CodeEraser GUI — the graph screen (batch 9 P18, user-ruled): the
// ce.graph-canvas document as a drawn map. File tier only; the
// layout is deterministic (no random seed — the same document draws
// the same picture); dead files and cycle members ride the alarm ramp;
// teal is selection only; magnitude is geometry (node radius =
// degree), never a printed number. Rendering only: every verdict on
// this screen came from the one core judgment the CLI also prints.
"use strict";

let graphDoc = null;
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
    graphDoc = await invoke("graphcanvas_report", { root: $("root").value });
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
    const p = map(gpos[i]), pos = file.pos, dead = file.verdict != null, cycle = pos && pos[3] > 1;
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
  c.title = i < 0 ? "" : graphDoc.files[i].path;
  c.classList.toggle("pick", i >= 0);
}

function graphClick(evt) {
  gsel = graphHit(evt);
  drawGraph();
  renderGraphAside(gsel);
}

function renderGraphAside(i) {
  const d = graphDoc, el = $("graph-detail");
  if (i < 0 || !d) {
    const c = d ? d.counts : { files: 0, edges: 0, dead: 0, cycles: 0 };
    let html = `<h2>${esc(tr("tabGraph"))}</h2>` +
      `<div class="row">${esc(tr("graphCounts", c.files, c.edges, c.dead, c.cycles))}</div>`;
    if (d && d.unresolvedSites > 0) html += `<div class="row zero">${esc(tr("graphUnresolved", d.unresolvedSites))}</div>`;
    if (d && d.degraded) html += `<div class="notice"><b>${esc(tr("degradedRun"))}</b><small>${esc(String(d.degraded))}</small></div>`;
    el.innerHTML = html;
    return;
  }
  const f = d.files[i], p = f.pos;
  let html = `<h2>${esc(String(f.path))}</h2>`;
  if (f.verdict != null) html += `<div class="row"><b>${esc(String(f.verdict))}</b></div><div class="row zero">${esc(String(f.why ?? ""))}</div>`;
  else html += `<div class="row">${esc(tr("graphAlive"))}</div>`;
  if (p != null) {
    html += `<div class="row">${esc(tr("graphInOut", p[0], p[1]))}</div>`;
    if (p[3] > 1) html += `<div class="row">${esc(tr("graphCycleOf", p[3]))}</div>`;
  }
  el.innerHTML = html;
}
