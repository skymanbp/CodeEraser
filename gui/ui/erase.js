// CodeEraser GUI — the erase screen (batch 4, plan ruling ②): the
// deterministic two-phase eraser's face over the SAME library road
// the CLI drives (one implementation, two faces — erase.md). The
// preview IS the plan: eraseable rows with their provenance, the
// advisory rows with their named reasons, and the unified diff the
// backend rendered from the hashed plan. Apply runs behind the
// contract preconditions and every refusal surfaces by name.
"use strict";

let eraseDoc = null;

(function bootErase() {
  i18nRefreshers.push(() => eraseDoc && renderErase());
  $("erase-preview").addEventListener("click", loadErase);
  $("erase-apply").addEventListener("click", applyErase);
})();

async function loadErase() {
  $("erase-preview").disabled = true;
  setStatus(tr("planning"), false);
  try {
    eraseDoc = await invoke("erase_preview", { root: $("root").value });
    renderErase();
    setStatus(eraseDoc.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("erase-preview").disabled = false;
  }
}

// The destructive phase — confirm first (the CLI's guard is the
// --apply flag; a button needs its own deliberate step), then the
// library preconditions do the real guarding and refuse by name.
async function applyErase() {
  const n = eraseDoc?.counts?.eraseable ?? 0;
  if (!n || !window.confirm(tr("applyConfirm", n))) return;
  $("erase-apply").disabled = true;
  setStatus(tr("applying"), false);
  try {
    const r = await invoke("erase_apply", { root: $("root").value });
    setStatus(tr("applied", r.applied), false);
    await loadErase(); // the post-apply plan is the convergence face
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("erase-apply").disabled = false;
  }
}

function renderErase() {
  $("empty-erase").hidden = true;
  const d = eraseDoc;
  $("erase-summary").innerHTML =
    `<span class="ok"><b>${d.counts.eraseable}</b> ${esc(tr("eraseable"))}</span>` +
    `<span><b>${d.counts.advisory}</b> ${esc(tr("advisory"))}</span>` +
    Object.entries(d.counts.out_of_class || {})
      .map(([k, v]) => `<span class="zero">${esc(k)} <b>${v}</b></span>`)
      .join("");
  $("erase-apply").hidden = d.counts.eraseable === 0;
  $("erase-rows").innerHTML = d.rows
    .map((r) => {
      const span = r.span ? `:${r.span[0]}-${r.span[1]}` : "";
      const mark = r.eraseable ? "✕" : "→";
      return (
        `<div class="erow ${r.eraseable ? "kill" : "adv"}">` +
        `<i>${mark}</i><b>${esc(r.path)}${span}</b> <em>${esc(r.class)}</em>` +
        `<span>${esc(r.eraseable ? tr("willErase") : r.reason)}</span>` +
        `<small>${esc(r.provenance)}</small></div>`
      );
    })
    .join("") || `<div class="erow adv">${esc(tr("nothingPlanned"))}</div>`;
  $("erase-diff").textContent = d.diff || "";
  $("erase-diff").hidden = !d.diff;
}
