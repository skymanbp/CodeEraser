// CodeEraser GUI — the score screen (batch 4): the ce.check-report
// document as a face — hero score over the effective scale, the
// seven verdict axes, and the ratchet's registers. Rendering
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
    // `--fail-under`, opt-in on every road — but CI arms 946 while
    // this screen could arm nothing, so one tree read pass here and
    // FAIL in the pipeline with nothing on screen to say why.
    const floor = $("check-floor").value.trim();
    checkDoc = await invoke("check_report", {
      root: $("root").value,
      floor: floor === "" ? null : posInt(floor, 0, 1000000),
    });
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
  // which conditions this verdict could have failed on, said out loud:
  // a pass with no floor armed is a weaker statement than a pass with
  // one, and the reader is entitled to know which they are looking at
  const floor = d.floor == null ? tr("floorOff") : tr("floorArmed", d.floor);
  // the number is NAMED: the structure screen also shows a score out
  // of 1000 and it is a different measure entirely (tree-scale entropy
  // vs this gate's seven axes plus the ratchet). The console has always
  // distinguished them — "check score" / "structure score" — and a
  // reader moving between two tabs deserves the same courtesy
  $("check-hero").innerHTML = notice +
    `<span id="check-score">${d.score}</span><small>/ ${scale}</small>` +
    `<small class="kind">${esc(tr("scoreCheck"))}</small>` +
    `<span class="verdict ${rt.fail ? "bad" : "ok"}">${verdict}</span>` +
    `<small class="floor">${esc(floor)}</small>` +
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
    // the fifth register (0.5.0): present exactly when the provenance
    // table rode, so its absence is shown as absence, not as zero
    ...(rt.dropped ? [[tr("dropped"), rt.dropped.length]] : []),
  ];
  $("check-ratchet").innerHTML =
    `<b>${esc(tr("ratchet"))}</b>` +
    regs.map(([k, v]) => row(k, v)).join("") +
    row(tr("candidates"), d.candidates.length) +
    "";
  // The join lattice's verdicts, reduced to a bare count until now.
  // Name and order both come off the wire (`joinSeverity` is the
  // core's own face); the only thing done here is tallying.
  const sev = new Map((d.joinSeverity || []).map(([c, s]) => [c, s]));
  const tally = new Map();
  for (const [, , code] of d.candidates) tally.set(code, (tally.get(code) ?? 0) + 1);
  const names = tr("joinVerdictNames");
  $("check-candidates").innerHTML = tally.size
    ? `<b>${esc(tr("byVerdict"))}</b>` +
      [...tally.entries()]
        .sort((a, b) => (sev.get(b[0]) ?? 0) - (sev.get(a[0]) ?? 0) || a[0] - b[0])
        .map(([code, n]) => row(names[code] ?? String(code), n))
        .join("")
    : "";
  // `failed` is the ratchet's NAMED register (the dense over rows are
  // wire identities, not prose) — the actionable half of the verdict
  $("check-named").innerHTML = (rt.failed || [])
    .map((name) => `<div class="row">▲ <b>${esc(String(name))}</b></div>`)
    .join("");
}
