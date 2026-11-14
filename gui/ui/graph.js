// CodeEraser GUI — the graph screen (batch 9 P18, user-ruled): the
// ce.graph-canvas document as a drawn map. File tier only; the
// layout is deterministic (no random seed — the same document draws
// the same picture); a root the project's gate blocks and cycle members
// ride the alarm ramp, every other unreferenced file is drawn hollow (a
// root's ring solid, a dependent's dashed); teal is selection only;
// magnitude is geometry (node radius = degree), never a printed number.
// Rendering only: every verdict on this screen came from the one core
// judgment the CLI also prints.
//
// The deadcode half of the reply is the document `ce deadcode --format
// json` prints for the same tree (plan v2.35 5C: the screen's judgment
// carries the gate through the deadcode face's one wire): its rows give
// each unreferenced file its root / dependent standing, a root's state
// and whether it blocks, and its `unmentioned` advisory rows sit beside
// the canvas. Both halves are joined here by path — a rendering join on
// a string the two documents share, never a verdict. Both arrive in ONE
// ce.graph-screen reply off one judgment (plan v2.30 step 5b item 31),
// so the rows a file lists describe the tree the map draws.
"use strict";

let graphDoc = null;
let deadDoc = null;
let advByPath = new Map();
let deadByPath = new Map();
let fallsByRoot = new Map();
let gpos = null, gsel = -1;

(function bootGraph() {
  i18nRefreshers.push(() => graphDoc && (renderGraphAside(gsel), drawGraph()));
  $("graph-load").addEventListener("click", loadGraph);
  const c = $("graphcanvas");
  c.addEventListener("mousemove", graphHover);
  c.addEventListener("click", graphClick);
  redrawers.graph = () => graphDoc && drawGraph();
})();

// Rows keyed by the path a row names: the advisory rows of a file, the
// unreferenced row of a file, and each root's dependents (the files
// whose `under` names it).
function groupBy(rows, key) {
  const by = new Map();
  for (const r of rows) {
    if (!by.has(key(r))) by.set(key(r), []);
    by.get(key(r)).push(r);
  }
  return by;
}

// One reply carries both documents off one judgment: the map and the
// deadcode rows commit together, so a failure never leaves the map on a
// new run beside rows from an old one. The status line names the
// canvas schema (what `show` returns).
function loadGraph() {
  return load("graph-load", tr("judging"), "graphscreen_report", { root: $("root").value }, (doc) => {
    graphDoc = doc.canvas;
    deadDoc = doc.deadcode;
    advByPath = groupBy(deadDoc.unmentioned ?? [], (a) => a.name);
    deadByPath = new Map((deadDoc.dead ?? []).map((d) => [d.name, d]));
    // one push per (dependent, root) pair: the core lets up to underCap
    // pairs through, so no per-pair copy of a root's list
    const pairs = (deadDoc.dead ?? []).flatMap((d) => (d.under ?? []).map((r) => [r, d.name]));
    fallsByRoot = new Map([...groupBy(pairs, (p) => p[0])].map(([r, ps]) => [r, ps.map((p) => p[1])]));
    $("empty-graph").hidden = true;
    gsel = -1;
    layoutGraph();
    drawGraph();
    renderGraphAside(-1);
    return graphDoc.schema;
  });
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

// The paths the selection is grouped with: a root's dependents (the
// files that fall with it), or a dependent's roots.
function linkedPaths(i) {
  if (i < 0) return new Set();
  const path = graphDoc.files[i].path, d = deadByPath.get(path);
  if (!d) return new Set();
  return new Set(d.root === false ? d.under ?? [] : fallsByRoot.get(path) ?? []);
}

// A node's look: a blocking root filled on the alarm ramp; any other
// unreferenced file hollow (panel fill, alarm ring), a root's ring
// solid and a dependent's dashed; cycle members and live files as the
// ramp has always drawn them.
function nodeLook(file, colors) {
  const d = deadByPath.get(file.path), cycle = file.cycle === true;
  if (file.verdict == null) {
    return { fill: cycle ? heat(1) : colors["--panel-2"], ring: cycle ? heat(2) : colors["--line"], dash: [] };
  }
  if (d && d.blocking === true) return { fill: heat(4), ring: heat(3), dash: [] };
  return { fill: colors["--panel-2"], ring: heat(3), dash: d && d.root === false ? [3, 2] : [] };
}

function drawNodes(ctx, map, colors) {
  const linked = linkedPaths(gsel);
  graphDoc.files.forEach((file, i) => {
    const p = map(gpos[i]), radius = nodeRadius(file), look = nodeLook(file, colors);
    ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, Math.PI * 2); ctx.fillStyle = look.fill; ctx.fill();
    ctx.setLineDash(look.dash);
    ctx.strokeStyle = linked.has(file.path) ? colors["--focus"] : look.ring;
    ctx.lineWidth = 1.5; ctx.stroke();
    ctx.setLineDash([]);
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

function graphSelect(i) {
  gsel = i;
  drawGraph();
  renderGraphAside(gsel);
}

function graphClick(evt) {
  graphSelect(graphHit(evt));
}
