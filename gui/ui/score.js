// CodeEraser GUI — the score screen (batch 4): the ce.check-report
// document as a face — hero score over the effective scale, the
// seven verdict axes, and the ratchet's four registers. Rendering
// only: the verdict came from the core, the FAIL/pass vocabulary
// stays machine-English exactly like the CLI's exit-code face.
"use strict";

let checkDoc = null;

(function bootScore() {
  i18nRefreshers.push(() => checkDoc && renderCheck());
  $("check-load").addEventListener("click", loadCheck);
})();

async function loadCheck() {
  $("check-load").disabled = true;
  setStatus(tr("judging"), false);
  try {
    checkDoc = await invoke("check_report", { root: $("root").value });
    renderCheck();
    setStatus(checkDoc.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("check-load").disabled = false;
  }
}

function renderCheck() {
  $("empty-score").hidden = true;
  const d = checkDoc;
  const scale = d.scoreScale ?? 1000;
  const rt = d.ratchet;
  // fail/pass stays English in both languages — it is the exit-code
  // vocabulary, not prose (the CLI console holds the same line)
  const verdict = rt.fail ? "FAIL" : "pass";
  const notice = d.degraded ? `<div class="notice"><b>${esc(tr("degradedRun"))}</b><small>${esc(String(d.degraded))}</small></div>` : "";
  $("check-hero").innerHTML = notice +
    `<span id="check-score">${d.score}</span><small>/ ${scale}</small>` +
    `<span class="verdict ${rt.fail ? "bad" : "ok"}">${verdict}</span>` +
    `<div id="check-bar"><div style="width:${(100 * d.score) / scale}%"></div></div>`;
  // the CHECK axes are the verdict family's seven (size…cycles), a
  // DIFFERENT vocabulary from structure's screen axes — the first
  // live run shipped "geometry 12" for what was the size axis
  $("check-axes").innerHTML = d.axes
    .map(
      ([c, p]) =>
        `<span${p === 0 ? ' class="zero"' : ""}>${esc(tr("checkAxisNames")[c] ?? String(c))} <b>${p}</b></span>`,
    )
    .join("");
  const regs = [
    [tr("added"), rt.added.length],
    [tr("removed"), rt.removed.length],
    [tr("overCeiling"), rt.over.length],
    [tr("toleranceDrawn"), rt.toleranceDrawn.length],
  ];
  $("check-ratchet").innerHTML =
    `<b>${esc(tr("ratchet"))}</b>` +
    regs.map(([k, v]) => row(k, v)).join("") +
    row(tr("candidates"), d.candidates.length) +
    "";
  // `failed` is the ratchet's NAMED register (the dense over rows are
  // wire identities, not prose) — the actionable half of the verdict
  $("check-named").innerHTML = (rt.failed || [])
    .map((name) => `<div class="row">▲ <b>${esc(String(name))}</b></div>`)
    .join("");
}
