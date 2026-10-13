// CodeEraser GUI — the query screen (plan v2.31 step 2): the code
// query family's face over the SAME documents `ce query` and `ce
// rules` print (ce.query-report / ce.rules-report). A question in CE
// Datalog answered by the core's query/1 — every goal as a table
// under its own column names, every answer labelled back to paths
// and names, its derivation rows under it when asked for (an
// assertion's always) — and the project's rules file judged the way
// `ce rules` does. Rendering only: no evaluation happens here, and a
// core without the family shows the document's named degraded
// posture, never a verdict of this screen's own.
"use strict";

let queryDoc = null;

(function bootQuery() {
  i18nRefreshers.push(() => queryDoc && renderQuery());
  $("query-run").addEventListener("click", () => loadQuery(false));
  $("query-rules").addEventListener("click", () => loadQuery(true));
  $("query-prelude").addEventListener("click", loadPrelude);
})();

// The text a question builds on (`ce query --prelude`, the same
// document the MCP `query` tool answers under `prelude`): the built-in
// prelude, then the program the core renders from `[rules.pack]`.
async function loadPrelude() {
  setStatus(tr("judging"), false);
  try {
    const d = await invoke("query_prelude", { root: $("root").value });
    queryDoc = null;
    $("empty-query").hidden = true;
    $("query-summary").innerHTML = "";
    $("query-rows").innerHTML =
      `<h3>${esc(tr("queryPreludeHead"))}</h3><pre>${esc(d.prelude)}</pre>` +
      `<h3>${esc(tr("queryPackHead"))}</h3>` +
      (d.pack === null ? `<div class="qline">${esc(tr("queryPackNone"))}</div>` : `<pre>${esc(d.pack)}</pre>`);
    setStatus(d.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  }
}

async function loadQuery(rules) {
  const text = $("query-text").value.trim();
  if (!rules && !text) {
    setStatus(tr("queryNeedsText"), true);
    return;
  }
  // the rules file (`--file`): empty = `[rules] file`, else ce.rules
  const file = $("query-file").value.trim() || null;
  const args = { root: $("root").value, why: $("query-why").checked, file };
  const btns = ["query-run", "query-rules"];
  const show = (d) => {
    queryDoc = d;
    renderQuery();
  };
  return rules
    ? load(btns, tr("judging"), "rules_report", args, show)
    : load(btns, tr("judging"), "query_report", { ...args, body: text }, show);
}

function renderQuery() {
  $("empty-query").hidden = true;
  const d = queryDoc;
  const c = d.counts;
  $("query-summary").innerHTML =
    `<span><b>${c.answers}</b> ${esc(tr("queryAnswers"))}</span>` +
    `<span class="${c.violations ? "bad" : "ok"}"><b>${c.violations}</b> ${esc(tr("queryViolations"))}</span>` +
    (d.errors.length ? `<span class="bad"><b>${d.errors.length}</b> ${esc(tr("queryErrors"))}</span>` : "") +
    (d.degraded ? `<span class="zero">${esc(tr("noJudgment", d.degraded))}</span>` : "");
  const errors = d.errors.map((e) => `<div class="qline bad">${esc(e.at)}: ${esc(e.what)}</div>`).join("");
  $("query-rows").innerHTML = errors + d.goals.map((g) => goalTable(d, g)).join("");
}

// One goal: its heading (a question's columns, or the assertion's
// name and verdict), a table of its answers, and under each answer
// the derivation rows the document carries, indented by depth. The
// answers and proof rows are grouped per goal and the depth walked here:
// deferred to the core lane — the query document should carry each
// goal's rows and each proof row's depth, and these walks then go.
function goalTable(d, g) {
  const rows = d.answers.map((a, i) => [a, i]).filter(([a]) => a.goal === g.goal);
  const title =
    g.kind === "assert"
      ? `assert ${esc(g.name)}(${esc(g.columns.join(", "))}) — ${rows.length ? `${rows.length} ${esc(tr("queryViolations"))}` : esc(tr("queryOk"))}`
      : `?- ${esc(g.columns.join(", "))} — ${rows.length} ${esc(tr("queryAnswers"))}`;
  if (!rows.length) return `<div class="qline${g.kind === "assert" ? " ok" : ""}">${title}</div>`;
  const head = g.columns.map((h, i) => `<th>${esc(h)} <em>${esc(g.sorts[i])}</em></th>`).join("");
  const width = g.columns.length;
  const body = rows
    .map(([a], nth) => {
      const cells = a.values.map((v) => `<td>${esc(v)}</td>`).join("");
      const proof = d.proof.filter((p) => p.goal === g.goal && p.answer === nth && p.parent >= 0);
      const chain = proof.length
        ? `<tr class="proof"><td colspan="${width}">${proof.map((p) => proofLine(p, proof)).join("")}</td></tr>`
        : "";
      return `<tr>${cells}</tr>${chain}`;
    })
    .join("");
  return `<h3>${title}</h3><table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
}

// One derivation node: `pred(args)` and whether a clause or a sent
// fact produced it, indented by its depth under the root.
function proofLine(p, rows) {
  let depth = 0;
  for (let at = p.parent; at >= 0; ) {
    const parent = rows.find((q) => q.node === at);
    if (!parent) break;
    depth += 1;
    at = parent.parent;
  }
  const by = p.rule < 0 ? tr("queryFact") : tr("queryClause", p.rule);
  return `<div class="proof-line" style="padding-left:${depth}em">${esc(p.pred)}(${esc(p.args.join(", "))}) <em>${esc(by)}</em></div>`;
}
