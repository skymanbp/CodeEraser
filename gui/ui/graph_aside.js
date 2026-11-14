// CodeEraser GUI — the graph screen's aside (split from graph.js at the
// house file line, plan v2.35 5C): the rows beside the map, off the
// deadcode half of the one ce.graph-screen reply — the gate, a root's
// state and the files that fall with it, a dependent's roots, the
// symbol-level advisory. Rendering only: every word and number came
// off the document; the screen joins its rows by path.
"use strict";

// How many names a root's "files that fall with it" row lists before
// it says how many more there are.
const FALLS_SHOWN = 12;

// A dependent's roots are links in the aside: a click selects that root.
(function bootGraphAside() {
  $("graph-detail").addEventListener("click", graphAsideClick);
})();

function graphAsideClick(evt) {
  const a = evt.target.closest("[data-path]");
  if (!a || !graphDoc) return;
  evt.preventDefault();
  graphSelect(graphDoc.files.findIndex((f) => f.path === a.dataset.path));
}

// The advisory's two states the rows cannot show for themselves: the
// core dropped the table (nothing judged at symbol level), or the
// producer cut the candidate set (the rows are a prefix). Both come
// off the report document's own flags, as the CLI prints them.
function advisoryNotices() {
  let html = "";
  if (deadDoc.unmentioned_dropped) html += `<div class="notice"><b>${esc(tr("advisoryDropped"))}</b></div>`;
  if (deadDoc.unmentioned_cut) html += `<div class="notice"><b>${esc(tr("advisoryCut"))}</b></div>`;
  return html;
}

// The whole-tree census of the advisory, by the core's code: counting
// is not judging, every code came off the wire.
function advisoryCensus() {
  const rows = deadDoc.unmentioned ?? [];
  const words = tr("advisoryWords");
  const by = new Map();
  for (const a of rows) by.set(a.code, (by.get(a.code) ?? 0) + 1);
  let html = `<div class="row zero">${esc(tr("advisoryHead", rows.length, advByPath.size))}</div>`;
  for (const [code, n] of by) html += row(words[code] ?? code, n);
  return html + advisoryNotices();
}

// The gate off the document's `gate` object, as the console reads it:
// one row per state that has a root (only when a comparison point rode
// — without one no state word is printed), then one row in three
// readings — nothing declared, declared and answered, declared and
// unanswered.
function gateRows() {
  const g = deadDoc.gate, c = deadDoc.counts, words = tr("stateWords");
  if (!g) return "";
  let html = "";
  if (g.comparison != null) {
    for (const s of ["standing", "orphaned", "born"]) if (c[s] > 0) html += row(words[s], c[s]);
  }
  const listed = (deadDoc.dead ?? []).length, states = g.block.map((s) => words[s] ?? s).join(", ");
  const say = g.block.length === 0 ? tr("gateListed", listed)
    : !g.answered ? tr("gateUnanswered", states)
    : g.blocked > 0 ? tr("gateBlocked", g.blocked, states) : tr("gateNoneBlocked", listed, states);
  return html + `<div class="row ${g.blocked > 0 || !g.answered ? "bad" : "zero"}">${esc(say)}</div>`;
}

function renderGraphAside(i) {
  const d = graphDoc, el = $("graph-detail");
  if (i < 0 || !d) {
    const c = d ? d.counts : { files: 0, edges: 0, cycles: 0 };
    const dc = deadDoc ? deadDoc.counts : {};
    let html = `<h2>${esc(tr("tabGraph"))}</h2>` +
      `<div class="row">${esc(tr("graphCounts", c.files, c.edges, dc.roots ?? 0, dc.dependents ?? 0, c.cycles))}</div>`;
    if (d && d.unresolvedSites > 0) html += `<div class="row zero">${esc(tr("graphUnresolved", d.unresolvedSites))}</div>`;
    if (d && d.degraded) html += `<div class="notice"><b>${esc(tr("degradedRun"))}</b><small>${esc(String(d.degraded))}</small></div>`;
    // the map draws an unreferenced file three ways; one quiet row says
    // which is which, when there is one to draw
    if (deadDoc && (deadDoc.dead ?? []).length) html += `<div class="row zero">${esc(tr("graphLegend"))}</div>`;
    if (d) html += gateRows() + advisoryCensus();
    el.innerHTML = html;
    return;
  }
  const f = d.files[i], p = f.pos;
  let html = `<h2>${esc(String(f.path))}</h2>`;
  if (f.verdict != null) html += verdictRows(f);
  else html += `<div class="row">${esc(tr("graphAlive"))}</div>`;
  if (p != null) {
    html += `<div class="row">${esc(tr("graphInOut", p[0], p[1]))}</div>`;
    if (f.cycle === true) html += `<div class="row">${esc(tr("graphCycleOf", p[3]))}</div>`;
  }
  el.innerHTML = html + advisoryOf(f.path);
}

// An unreferenced file's rows: the verdict code and the core's reason by
// code (ce.graph-canvas/0.4.0; an unknown code shows as its number), its
// root or dependent standing, then the trust column (2.32.0) the console
// prints beside the verdict — the same number decides whether `ce
// erase` may act on the row, so a verdict shown without it is a
// stronger claim than the core made.
function verdictRows(f) {
  let html = `<div class="row"><b>${esc(String(f.verdict))}</b></div><div class="row zero">${esc(tr("deadWhy")[f.whyCode] ?? String(f.whyCode))}</div>`;
  const d = deadByPath.get(f.path);
  if (d) html += d.root === false ? underRows(d) : rootRows(d);
  if (f.conf != null) {
    const word = tr("trustNames")[f.conf] ?? String(f.conf);
    html += `<div class="row ${f.conf === 0 ? "bad" : "zero"}">${esc(tr("trust"))}: <b>${esc(word)}</b></div>`;
  }
  return html;
}

// A root: its state (when a comparison point rode), a marked row when
// the gate blocks it, and the files that fall with it — the count, then
// the names, cut at FALLS_SHOWN; nothing when none falls, as the console.
function rootRows(d) {
  let html = "";
  if (deadDoc.gate && deadDoc.gate.comparison != null && d.state != null) {
    html += row(tr("rootState"), tr("stateWords")[d.state] ?? d.state);
  }
  if (d.blocking === true) html += `<div class="row bad"><b>${esc(tr("rootBlocks"))}</b></div>`;
  const falls = fallsByRoot.get(d.name) ?? [];
  if (!falls.length) return html;
  html += `<div class="row">${esc(tr("graphFalls", falls.length))}</div>`;
  for (const n of falls.slice(0, FALLS_SHOWN)) html += `<div class="row"><span class="dir">${esc(n)}</span></div>`;
  if (falls.length > FALLS_SHOWN) html += `<div class="row zero">${esc(tr("moreFiles", falls.length - FALLS_SHOWN))}</div>`;
  return html;
}

// A dependent: its roots by name, each a link that selects that root.
function underRows(d) {
  const links = (d.under ?? []).map((r) => `<a href="#" data-path="${esc(r)}">${esc(r)}</a>`).join(", ");
  return `<div class="row">${esc(tr("graphUnder"))} ${links}</div>`;
}

// The file's own unmentioned declarations (the symbol-level
// drill-down): line, name and the core's code word, the code's
// reading on the tooltip. A file with none says so in one quiet row
// rather than showing nothing — "no advisory" and "not loaded" must
// not look alike.
function advisoryOf(path) {
  const rows = advByPath.get(path) ?? [];
  const words = tr("advisoryWords");
  let html = `<div class="row zero">${esc(tr("advisoryHead", rows.length, rows.length ? 1 : 0))}</div>`;
  for (const a of rows) {
    html += `<div class="row" title="${esc(String(a.why))}"><span class="dir">${esc(String(a.line))}</span> ` +
      `<b>${esc(String(a.symbol))}</b> <span class="dir">${esc(words[a.code] ?? String(a.code))}</span></div>`;
  }
  return html + advisoryNotices();
}
