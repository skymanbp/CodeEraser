// CodeEraser GUI — the erase screen (batch 4, plan ruling ②): the
// deterministic two-phase eraser's face over the SAME library road
// the CLI drives (one implementation, two faces — erase.md). The
// preview IS the plan: eraseable rows with their provenance, the
// advisory rows with their named reasons, and the unified diff the
// backend rendered from the hashed plan. Apply runs behind the
// contract preconditions and every refusal surfaces by name. The
// audit log section (plan v2.29 step 9, O50) reads back what apply
// appended — the same document `ce erase --log` prints, never
// written from here.
"use strict";

let eraseDoc = null;
let eraseLogDoc = null;

(function bootErase() {
  i18nRefreshers.push(() => {
    if (eraseDoc) renderErase();
    if (eraseLogDoc) renderEraseLog();
  });
  $("erase-preview").addEventListener("click", loadErase);
  $("erase-apply").addEventListener("click", applyErase);
  $("erase-log").addEventListener("click", loadEraseLog);
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
    if (eraseLogDoc) await loadEraseLog(); // an open log shows the new records
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("erase-apply").disabled = false;
  }
}

async function loadEraseLog() {
  $("erase-log").disabled = true;
  setStatus(tr("readingLog"), false);
  try {
    eraseLogDoc = await invoke("erase_log_report", { root: $("root").value });
    renderEraseLog();
    setStatus(eraseLogDoc.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("erase-log").disabled = false;
  }
}

function renderErase() {
  $("empty-erase").hidden = true;
  const d = eraseDoc;
  // an out-of-class kind carries the family command that owns it
  // (O24, `families` since ce.erase-plan/0.3.0) — the same table the
  // console sentence reads
  const families = d.families || {};
  $("erase-summary").innerHTML =
    `<span class="ok"><b>${d.counts.eraseable}</b> ${esc(tr("eraseable"))}</span>` +
    `<span><b>${d.counts.advisory}</b> ${esc(tr("advisory"))}</span>` +
    Object.entries(d.counts.out_of_class || {})
      .map(([k, v]) => {
        const cmd = families[k] ? ` → <code>${esc(families[k])}</code>` : "";
        return `<span class="zero">${esc(k)} <b>${v}</b>${cmd}</span>`;
      })
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

// The stamp the console prints for the same integer (erase::log::utc_stamp):
// seconds, UTC, no fraction — three faces, one reading of ts_ms.
function logStamp(ms) {
  return new Date(ms).toISOString().replace(/\.\d{3}Z$/, "Z");
}

function renderEraseLog() {
  $("empty-erase").hidden = true;
  const d = eraseLogDoc;
  const el = $("erase-log-rows");
  el.hidden = false;
  const head =
    `<div class="erow adv"><i>≡</i><b>${esc(tr("logHead", d.counts.rows, d.counts.unreadable))}</b>` +
    `<em>${esc(d.log)}</em><span></span><small></small></div>`;
  const rows = d.rows
    .map((r) => {
      const span = r.span ? `:${r.span[0]}-${r.span[1]}` : "";
      return (
        `<div class="erow kill"><i>✕</i><b>${esc(r.path)}${span}</b> <em>${esc(r.class)}</em>` +
        `<span>${esc(logStamp(r.ts_ms))}</span>` +
        `<small>${esc(r.provenance)} · plan ${esc(r.plan)}</small></div>`
      );
    })
    .join("");
  const bad = d.unreadable
    .map(
      (u) =>
        `<div class="erow adv"><i>!</i><b>${esc(tr("logUnreadable", u.line))}</b>` +
        `<em></em><span>${esc(u.why)}</span><small></small></div>`,
    )
    .join("");
  // two spelled-out tr() calls: the i18n gate harvests literal keys
  const why = d.present ? tr("logEmpty") : tr("logAbsent");
  const none = rows ? "" : `<div class="erow adv">${esc(why)}</div>`;
  el.innerHTML = head + rows + none + bad;
}
