// CodeEraser GUI — the clone-merge document in the reports hub (plan
// v2.31 step 7): the SAME document `ce merge` prints (ce.merge-report),
// registered as a hub family with its own renderer because a group
// nests — its members, and per parameter every member's text at it.
// The counts and the groups not sent lead as chips; then one card per
// group: its head (family, fragment, members, parameters, savings,
// feasible or why not), its members (with the run each sent when a
// trim made it shorter than the member) and the parameter table — a row
// per parameter, a column per member, the text monospaced and cut
// with the whole of it in the cell's title. Rendering only: the
// alignment, the parameters, the kept member, the savings and the
// feasibility are the core's merge/1, and a degraded document is one
// named line.
"use strict";

registerHub("merge", { cmd: "merge_report", render: renderMerge });

// A parameter's text in its cell: 40 characters on screen.
const MERGE_CELL = 40;

function renderMerge(d) {
  const chip = (k, v) => `<span>${esc(k)} <b>${esc(String(v))}</b></span>`;
  $("hub-chips").innerHTML = [
    ...Object.entries(d.counts).map(([k, v]) => chip(`counts.${k}`, v)),
    ...Object.entries(d.unsendable).map(([k, v]) => chip(`${tr("mergeUnsendable")} ${k}`, v)),
  ].join("");
  if (d.degraded) {
    $("hub-tables").innerHTML = `<div class="row zero">${esc(tr("mergeDegraded", d.degraded))}</div>`;
    return;
  }
  $("hub-tables").innerHTML = d.groups.map(mergeCard).join("");
}

// The reason's own words; the document's code name stays in the title.
const mergeReason = (r) =>
  ({
    ok: tr("mergeReasonOk"),
    position: tr("mergeReasonPosition"),
    type: tr("mergeReasonType"),
    spans_statements: tr("mergeReasonSpans"),
    too_many_params: tr("mergeReasonParams"),
    no_savings: tr("mergeReasonSavings"),
  })[r] ?? r;

// A member's lines: its clone-family span, and the run it sent when a
// trim made that shorter (the lines the core priced).
function mergeLines(m) {
  const span = `${m.lines[0]}–${m.lines[1]}`;
  const same = m.run[0] === m.lines[0] && m.run[1] === m.lines[1];
  return same ? span : `${span} <small>(${esc(tr("mergeRun"))} ${m.run[0]}–${m.run[1]})</small>`;
}

function mergeHead(g) {
  const family = g.fragment ? `${g.family} · ${tr("mergeFragment")}` : g.family;
  const verdict = g.feasible ? tr("mergeFeasible") : `${tr("mergeInfeasible")} (${mergeReason(g.reason)})`;
  return (
    `<h3>${esc(tr("mergeGroup"))} ${g.group} <small>${esc(family)}</small></h3>` +
    `<div class="stat">` +
    `<span>${esc(tr("mergeFamily"))} <b>${esc(g.family)}</b></span>` +
    `<span>${esc(tr("mergeMembers"))} <b>${g.members.length}</b></span>` +
    `<span>${esc(tr("mergeParams"))} <b>${g.params}</b></span>` +
    `<span>${esc(tr("mergeSavings"))} <b>${g.savings}</b></span>` +
    `<span title="${esc(g.reason)}">${esc(tr("mergeReason"))} <b>${esc(verdict)}</b></span>` +
    `</div>`
  );
}

function mergeMembersTable(g) {
  const rows = g.members.map(
    (m, i) =>
      `<tr><td class="num">m${i}${i === g.kept ? " ★" : ""}</td>` +
      `<td><code>${esc(m.unit ?? m.path)}</code></td>` +
      `<td class="num">${mergeLines(m)}</td></tr>`,
  );
  return `<table><tbody>${rows.join("")}</tbody></table>`;
}

// One member's text at a parameter: the hole's source on that member,
// "" on an empty side.
function mergeCell(v) {
  const text = v.text;
  const shown = text.length > MERGE_CELL ? `${text.slice(0, MERGE_CELL)}…` : text;
  return `<td title="${esc(text)}"><code>${esc(shown)}</code></td>`;
}

function mergeHolesTable(g) {
  if (!g.holes.length) return "";
  const head = g.members.map((_, i) => `<th>m${i}</th>`).join("");
  const rows = g.holes.map(
    (p) => `<tr><td class="num">${p.param + 1}</td>${p.values.map(mergeCell).join("")}</tr>`,
  );
  return (
    `<h4>${esc(tr("mergeHoles"))}</h4>` +
    `<table><thead><tr><th>#</th>${head}</tr></thead><tbody>${rows.join("")}</tbody></table>`
  );
}

function mergeCard(g) {
  return `<div class="merge-card">${mergeHead(g)}${mergeMembersTable(g)}${mergeHolesTable(g)}</div>`;
}
