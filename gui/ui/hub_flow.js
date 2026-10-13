// CodeEraser GUI — the flow family in the diagnostics hub (plan v2.31
// step 5): the dead code inside functions, the SAME document
// `ce flow --format json` prints. The hub's generic renderer would
// flatten it; this one keeps what the document says: the counts as
// chips, four kind chips that narrow the findings table — the shown
// kinds ride the ask as `kind`, as `--kind` does, so the document lists
// them and the counts stay whole — each finding with its lines,
// variable and judged / advisory mark, the units left unjudged with
// their reasons, and a degraded document's reason in one line. The
// findings are the core's; this file only lays them out.
"use strict";

// the kinds as the core's flow catalogue lists them, `{name, en, zh}`
// (plan v2.32 step 5, R8): read once, after the first document, so the
// chips and the kind column carry the core's labels and no map of ours
let flowKinds = null;
// the kinds shown; a click on a kind chip toggles it and asks again
let flowShown = null;

registerHub("flow", { cmd: "flow_report", render: renderFlow, args: flowArgs });

// `kind`: the shown kinds, or absent (every kind) while all are shown
// or before the catalogue is read.
function flowArgs() {
  const some = flowKinds && flowShown.size < flowKinds.length;
  return { kind: some ? [...flowShown] : null };
}

function flowLabel(name) {
  const k = (flowKinds ?? []).find((r) => r.name === name);
  return k ? k[ceLang] : name;
}

function renderFlow(d) {
  if (!flowKinds) {
    invoke("flow_kinds", { root: $("root").value })
      .then((rows) => {
        flowKinds = rows;
        flowShown = new Set(rows.map((r) => r.name));
        renderFlow(d);
      })
      .catch((e) => setStatus(String(e), true));
    return;
  }
  const counts = Object.entries(d.counts ?? {})
    .map(([k, v]) => hubChip(k, v))
    .join("");
  const kinds = flowKinds
    .map(({ name }) => {
      const on = flowShown.has(name) ? " on" : "";
      return `<button class="flow-kind${on}" data-kind="${esc(name)}">${esc(flowLabel(name))}</button>`;
    })
    .join("");
  $("hub-chips").innerHTML = counts + kinds;
  document.querySelectorAll("#hub-chips .flow-kind").forEach((b) =>
    b.addEventListener("click", () => {
      const k = b.dataset.kind;
      if (flowShown.has(k)) flowShown.delete(k);
      else flowShown.add(k);
      // no kind shown lists nothing — nothing to ask the core for
      if (flowShown.size) loadHub();
      else renderFlow(d);
    }),
  );
  // the document lists the shown kinds (`kind` rode the ask); with no
  // kind chosen nothing is listed
  const findings = flowShown.size ? d.findings ?? [] : [];
  $("hub-tables").innerHTML = flowDegraded(d) + flowFindings(findings) + flowRefused(d.refused ?? []);
}

function flowDegraded(d) {
  return d.degraded ? `<div class="row zero">${esc(tr("noJudgment", d.degraded))}</div>` : "";
}

function flowFindings(rows) {
  if (!rows.length) return `<div class="row zero">${esc(tr("flowNoFinding"))}</div>`;
  const line = (f) => {
    const lines = f.line === f.lineEnd ? `${f.line}` : `${f.line}–${f.lineEnd}`;
    const mark = f.judged ? tr("flowJudged") : tr("flowAdvisory");
    const cells = [f.path, f.unit, flowLabel(f.kind), lines, f.var ?? "—", mark];
    return `<tr>${cells.map((c) => `<td>${esc(String(c))}</td>`).join("")}</tr>`;
  };
  return hubTable("findings", rows.length, ["path", "unit", "kind", "line", "var", "judged"], rows, line);
}

function flowRefused(rows) {
  if (!rows.length) return "";
  const line = (r) => `<tr><td>${esc(r.path)}</td><td>${esc(r.unit)}</td><td>${esc(r.reason)}</td></tr>`;
  return hubTable(tr("flowRefused"), rows.length, ["path", "unit", "reason"], rows, line);
}
