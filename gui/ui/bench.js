// CodeEraser GUI — the bench screen (batch 5): the compiled-in
// bench.json rendered as itself. The series pivots to one row per
// metric with a column per version (the honest cross-release read);
// frozen points list value + source — a number without its source
// is exactly what this dashboard exists to forbid. Loads once on
// boot: the document is baked into the binary, no root involved.
"use strict";

let benchDoc = null;

(function bootBench() {
  i18nRefreshers.push(() => benchDoc && renderBench());
  invoke("bench_doc")
    .then((d) => {
      benchDoc = d;
      renderBench();
    })
    .catch(() => {});
})();

function renderBench() {
  const d = benchDoc;
  const versions = [...new Set(d.rows.map((r) => r.version))];
  const metrics = [...new Set(d.rows.map((r) => r.metric))];
  const cell = (m, v) => {
    const r = d.rows.find((x) => x.metric === m && x.version === v);
    return r ? `${r.p50}<small>/${r.p95}</small>` : "·";
  };
  $("bench-series").innerHTML =
    `<h3>${esc(tr("benchSeries"))}</h3>` +
    `<table><thead><tr><th>${esc(tr("benchMetric"))}</th>` +
    versions.map((v) => `<th>v${esc(v)}</th>`).join("") +
    `</tr></thead><tbody>` +
    metrics
      .map(
        (m) =>
          `<tr><td>${esc(m)}</td>` +
          versions.map((v) => `<td>${cell(m, v)}</td>`).join("") +
          `</tr>`,
      )
      .join("") +
    `</tbody></table>` +
    `<div class="row zero">${esc(tr("benchUnit"))}</div>`;
  $("bench-frozen").innerHTML =
    `<h3>${esc(tr("benchFrozen"))}</h3>` +
    d.frozen
      .map(
        (f) =>
          `<div class="erow adv"><i>❄</i><b>${esc(f.metric)}</b>` +
          `<em>${esc(f.value)}</em>` +
          `<span>${esc(f.detail ?? "")}</span>` +
          `<small>${esc(f.source)}${f.epoch ? " · " + esc(f.epoch) : ""}</small></div>`,
      )
      .join("");
}
