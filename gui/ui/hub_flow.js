// CodeEraser GUI — the flow family in the diagnostics hub (plan v2.31
// step 5): the dead code inside functions, the SAME document
// `ce flow --format json` prints. The hub's generic renderer would
// flatten it; this one keeps what the document says: the counts as
// chips, four kind chips that filter the findings table (the counts
// stay whole, as `--kind` leaves them), each finding with its lines,
// variable and judged / advisory mark, the units left unjudged with
// their reasons, and a degraded document's reason in one line. The
// findings are the core's; this file only lays them out.
"use strict";

const FLOW_KINDS = {
  unreachable: () => tr("flowKindUnreachable"),
  dead_store: () => tr("flowKindDeadStore"),
  unused_local: () => tr("flowKindUnusedLocal"),
  unused_param: () => tr("flowKindUnusedParam"),
};

// the kinds shown; a click on a kind chip toggles it
const flowShown = new Set(Object.keys(FLOW_KINDS));

registerHub("flow", { cmd: "flow_report", render: renderFlow });

function renderFlow(d) {
  const counts = Object.entries(d.counts ?? {})
    .map(([k, v]) => `<span>${esc(k)} <b>${esc(String(v))}</b></span>`)
    .join("");
  const kinds = Object.entries(FLOW_KINDS)
    .map(([k, label]) => {
      const on = flowShown.has(k) ? " on" : "";
      return `<button class="chip flow-kind${on}" data-kind="${k}">${esc(label())}</button>`;
    })
    .join("");
  $("hub-chips").innerHTML = counts + kinds;
  document.querySelectorAll("#hub-chips .flow-kind").forEach((b) =>
    b.addEventListener("click", () => {
      const k = b.dataset.kind;
      if (flowShown.has(k)) flowShown.delete(k);
      else flowShown.add(k);
      renderFlow(d);
    }),
  );
  $("hub-tables").innerHTML = flowDegraded(d) + flowFindings(d.findings ?? []) + flowRefused(d.refused ?? []);
}

function flowDegraded(d) {
  return d.degraded ? `<div class="row zero">${esc(tr("flowDegraded", d.degraded))}</div>` : "";
}

function flowFindings(all) {
  const rows = all.filter((f) => flowShown.has(f.kind));
  if (!rows.length) return `<div class="row zero">${esc(tr("flowNoFinding"))}</div>`;
  const body = rows
    .slice(0, HUB_ROW_CAP)
    .map((f) => {
      const lines = f.line === f.lineEnd ? `${f.line}` : `${f.line}–${f.lineEnd}`;
      const mark = f.judged ? tr("flowJudged") : tr("flowAdvisory");
      const kind = FLOW_KINDS[f.kind] ? FLOW_KINDS[f.kind]() : f.kind;
      const cells = [f.path, f.unit, kind, lines, f.var ?? "—", mark];
      return `<tr>${cells.map((c) => `<td>${esc(String(c))}</td>`).join("")}</tr>`;
    })
    .join("");
  const cap =
    rows.length > HUB_ROW_CAP ? `<div class="row zero">${esc(tr("rowsCapped", HUB_ROW_CAP, rows.length))}</div>` : "";
  const head = ["path", "unit", "kind", "line", "var", "judged"];
  return (
    `<h3>findings <small>${rows.length}</small></h3>` +
    `<table><thead><tr>${head.map((h) => `<th>${esc(h)}</th>`).join("")}</tr></thead>` +
    `<tbody>${body}</tbody></table>` +
    cap
  );
}

function flowRefused(rows) {
  if (!rows.length) return "";
  const body = rows
    .map((r) => `<tr><td>${esc(r.path)}</td><td>${esc(r.unit)}</td><td>${esc(r.reason)}</td></tr>`)
    .join("");
  return (
    `<h3>${esc(tr("flowRefused"))} <small>${rows.length}</small></h3>` +
    `<table><thead><tr><th>path</th><th>unit</th><th>reason</th></tr></thead>` +
    `<tbody>${body}</tbody></table>`
  );
}
