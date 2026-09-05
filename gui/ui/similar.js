// CodeEraser GUI — the similar screen (plan v2.29 step 6): the
// same-role advisor's face over the SAME document `ce similar` prints
// (ce.similar-report). A unit at file:line, or a text; the candidates
// in the order the core answered, the six-channel evidence row per
// candidate, the core's role bit, and — under widen — the associative
// view's rows tagged. Rendering only: no ranking and no conjunction
// happen here; a core without the family shows the document's named
// degraded posture and a null role column, never a verdict of this
// screen's own.
"use strict";

let similarDoc = null;

(function bootSimilar() {
  i18nRefreshers.push(() => similarDoc && renderSimilar());
  $("similar-load").addEventListener("click", loadSimilar);
})();

async function loadSimilar() {
  const at = $("similar-at").value.trim();
  const text = $("similar-text").value.trim();
  if (!at && !text) {
    setStatus(tr("similarNeedsQuery"), true);
    return;
  }
  $("similar-load").disabled = true;
  setStatus(tr("judging"), false);
  try {
    // `at` wins when both are typed: a unit is the sharper ask
    similarDoc = await invoke("similar_report", {
      root: $("root").value,
      at: at || null,
      text: at ? null : text,
      widen: $("similar-widen").checked,
    });
    renderSimilar();
    setStatus(similarDoc.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("similar-load").disabled = false;
  }
}

function renderSimilar() {
  $("empty-similar").hidden = true;
  const d = similarDoc;
  const c = d.counts;
  $("similar-summary").innerHTML =
    `<span>${esc(d.query.label)}</span>` +
    `<span><b>${c.candidates}</b> ${esc(tr("candidates"))}</span>` +
    `<span class="ok"><b>${c.role}</b> ${esc(tr("sameRole"))}</span>` +
    (d.query.widen ? `<span><b>${c.widened}</b> ${esc(tr("widenedRows"))}</span>` : "") +
    (d.degraded ? `<span class="zero">${esc(tr("similarDegraded", d.degraded))}</span>` : "");
  const head = ["at", "key", tr("evidence"), tr("role"), tr("score")]
    .map((h) => `<th>${esc(h)}</th>`)
    .join("");
  const body = d.candidates.map(similarRow).join("");
  $("similar-rows").innerHTML = d.candidates.length
    ? `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`
    : `<div class="erow adv">${esc(tr("noCandidates"))}</div>`;
}

// One candidate: where, what (tagged when the widened query alone
// reached it), the hits in wire order, the role word (`?` = the core
// did not judge), the score's integer part.
function similarRow(r) {
  const tag = r.widened ? ` <em>${esc(tr("widenedTag"))}</em>` : "";
  const role = r.role === null ? "?" : r.role ? tr("sameRole") : "—";
  const hits = ["N", "P", "C", "D", "S", "L"].map((l, i) => `${l}${r.hits[i]}`).join(" ");
  return (
    `<tr class="${r.role ? "role" : ""}">` +
    `<td>${esc(r.at)}</td><td>${esc(r.key)}${tag}</td>` +
    `<td class="num">${esc(hits)}</td><td>${esc(role)}</td><td class="num">${r.score}</td></tr>`
  );
}
