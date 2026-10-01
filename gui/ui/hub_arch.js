// CodeEraser GUI — the architecture document in the reports hub (plan
// v2.31 step 9): the SAME document `ce arch` prints (ce.arch-report),
// registered as a hub family with its own renderer because its rows
// nest — a cut carries the file references behind it, a cluster its
// files. The counts lead as chips, the way every hub family's do;
// then the layers from the top level down, the cuts with each file
// reference under its arc, the misplaced files, the clusters, the
// impact walk when paths were given, and the directory metrics.
// Rendering only: the cut arcs, the levels, the clusters and the
// depths are the core's arch/1, and a degraded document is one named
// line, never a picture of this screen's own.
"use strict";

registerHub("arch", { cmd: "arch_report", paths: "impact", render: renderArch });

function renderArch(d) {
  $("hub-chips").innerHTML = Object.entries(d.counts)
    .map(([k, v]) => `<span>counts.${esc(k)} <b>${esc(String(v))}</b></span>`)
    .join("");
  if (d.degraded) {
    $("hub-tables").innerHTML = `<div class="row zero">${esc(tr("archDegraded", d.degraded))}</div>`;
    return;
  }
  $("hub-tables").innerHTML = [
    archLayers(d),
    archCuts(d),
    archMisplaced(d),
    archClusters(d),
    d.counts.focus ? archImpact(d) : "",
    archMetrics(d),
  ].join("");
}

// A directory as the console spells it: the root reads ".".
const archDir = (dir) => (dir === "" ? "." : dir);
const archCell = (v, num) => `<td${num ? ' class="num"' : ""}>${esc(String(v))}</td>`;

// One titled table: a heading with its row count, a header row, rows.
function archTable(title, count, head, rows) {
  return (
    `<h3>${esc(title)} <small>${count}</small></h3>` +
    `<table><thead><tr>${head.map((h) => `<th>${esc(h)}</th>`).join("")}</tr></thead>` +
    `<tbody>${rows.join("")}</tbody></table>`
  );
}

// One band per level, the highest first, its directories as chips.
function archLayers(d) {
  const by = new Map();
  for (const l of d.layers) by.set(l.level, [...(by.get(l.level) ?? []), archDir(l.dir)]);
  const bands = [...by.keys()]
    .sort((a, b) => b - a)
    .map((level) => `<tr>${archCell(level, true)}<td>${by.get(level).map((x) => `<code>${esc(x)}</code>`).join(" ")}</td></tr>`);
  return archTable(tr("archLayers"), bands.length, [tr("archLevel"), ""], bands);
}

// Every cut arc, and under it the file references it folds.
function archCuts(d) {
  if (!d.cuts.length) return `<h3>${esc(tr("archCuts"))}</h3><div class="row zero">${esc(tr("archNoCycles"))}</div>`;
  const rows = d.cuts.flatMap((c) => [
    `<tr><td>${esc(archDir(c.from))} → ${esc(archDir(c.to))}</td>${archCell(c.refs, true)}<td>${esc(c.exact ? tr("archExact") : tr("archGreedy"))}</td></tr>`,
    ...c.files.map((f) => `<tr><td>&nbsp;&nbsp;↳ ${esc(f.from)} → ${esc(f.to)}</td>${archCell(f.refs, true)}<td></td></tr>`),
  ]);
  return archTable(tr("archCuts"), d.cuts.length, ["", tr("archRefs"), ""], rows);
}

function archMisplaced(d) {
  const rows = d.misplaced.map((m) => `<tr>${archCell(m.path)}${archCell(archDir(m.dir))}${archCell(archDir(m.majority))}</tr>`);
  return archTable(tr("archMisplaced"), rows.length, ["", "", tr("archMajority")], rows);
}

// A cluster with its majority directory and its size; the files
// follow on one line each under it.
function archClusters(d) {
  const rows = d.clusters.flatMap((c) => [
    `<tr>${archCell(c.cluster, true)}${archCell(archDir(c.majority))}${archCell(c.files.length, true)}</tr>`,
    ...c.files.map((f) => `<tr><td></td><td colspan="2">&nbsp;&nbsp;↳ ${esc(f)}</td></tr>`),
  ]);
  return archTable(tr("archClusters"), d.clusters.length, ["", tr("archMajority"), ""], rows);
}

function archImpact(d) {
  const rows = d.impact.map((i) => `<tr>${archCell(i.path)}${archCell(i.depth, true)}</tr>`);
  return archTable(tr("archImpact"), rows.length, ["", tr("archDepth")], rows);
}

// fan-in, fan-out and the per-mille instability; "-" where no arc
// touches the directory.
function archMetrics(d) {
  const rows = d.metrics.map(
    (m) =>
      `<tr>${archCell(archDir(m.dir))}${archCell(m.fanIn, true)}${archCell(m.fanOut, true)}${archCell(m.instability ?? "-", true)}</tr>`,
  );
  return archTable(tr("archMetrics"), rows.length, ["", tr("archFanIn"), tr("archFanOut"), tr("archInstability")], rows);
}
