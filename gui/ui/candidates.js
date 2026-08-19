// CodeEraser GUI — deletion-candidate browser (M7-P4, i18n'd
// M8-G3a). Renders the ce.join-report rows (three-signal file pairs
// + unit pairs) and the ce.dedup-report block list, in DOCUMENT
// ORDER — the list is the CLI's own report rows through the same
// report_json throats, and no verdict or ranking is derived here.
"use strict";

let joinDoc = null;
let dedupDoc = null;

function candBoot() {
  $("cand-load").addEventListener("click", loadCandidates);
  i18nRefreshers.push(() => joinDoc && dedupDoc && renderCandidates());
}

async function loadCandidates() {
  const days = posInt($("cand-days").value, 1, 14);
  $("cand-load").disabled = true;
  setStatus(tr("joining"), false);
  try {
    // Fetch both documents, then commit both — a failure must not
    // leave one screen-half stale against the other's new run.
    const root = $("root").value;
    const [j, d] = await Promise.all([
      invoke("join_report", { root, days }),
      invoke("dedup_report", { root }),
    ]);
    joinDoc = j;
    dedupDoc = d;
    renderCandidates();
    setStatus(joinDoc.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("cand-load").disabled = false;
  }
}

function renderCandidates() {
  $("empty-candidates").hidden = true;
  const parts = [];
  parts.push(`<h2>${tr("pairsHead", joinDoc.files.length, joinDoc.days, joinDoc.commits)}</h2>`);
  if (joinDoc.degraded) parts.push(`<p class="err">${esc(tr("degraded", joinDoc.degraded))}</p>`);
  joinDoc.files.forEach((f, i) => {
    parts.push(
      `<div class="cand" data-kind="file" data-i="${i}">` +
      `<span class="pair"><b>${esc(f.a)}</b> ↔ <b>${esc(f.b)}</b></span>` +
      `<span>${tr("blockTokens", f.blocks, f.tokens)}</span></div>`
    );
  });
  parts.push(`<h2>${tr("unitPairs")} — ${joinDoc.units.length}</h2>`);
  joinDoc.units.forEach((u, i) => {
    parts.push(
      `<div class="cand" data-kind="unit" data-i="${i}">` +
      `<span class="pair"><b>${esc(u.a.path)}#${esc(u.a.key)}</b> ↔ <b>${esc(u.b.path)}#${esc(u.b.key)}</b></span>` +
      `<span>${tr("tokensOnly", u.tokens)}</span></div>`
    );
  });
  parts.push(`<h2>${tr("cloneBlocks")} — ${dedupDoc.blocks.length}</h2>`);
  dedupDoc.blocks.forEach((b, i) => {
    parts.push(
      `<div class="cand" data-kind="block" data-i="${i}">` +
      `<span class="pair"><b>${esc(b.a_file)}:${b.a_start}-${b.a_end}</b> ↔ ` +
      `<b>${esc(b.b_file)}:${b.b_start}-${b.b_end}</b></span>` +
      `<span>${tr("tokensOnly", b.tokens)}</span></div>`
    );
  });
  const list = $("cand-list");
  list.innerHTML = parts.join("");
  list.querySelectorAll(".cand").forEach((el) =>
    el.addEventListener("click", () => {
      list.querySelectorAll(".cand.sel").forEach((s) => s.classList.remove("sel"));
      el.classList.add("sel");
      candDetail(el.dataset.kind, Number(el.dataset.i));
    }));
}

const pos = (p) =>
  p === null ? tr("posNull") : `in ${p[0]} · out ${p[1]} · scc ${p[2]}×${p[3]} · reach ${p[4]}`;
const churnStr = (c) => `+${c.appended} / ~${c.rewrote}`;

// row() escapes its value itself — everything passed in stays RAW
// (pre-escaping here double-escaped caveats and paths to visible
// entities).
function candDetail(kind, i) {
  const rows = [];
  if (kind === "file") {
    const f = joinDoc.files[i];
    rows.push(`<h2>${esc(f.a)} ↔ ${esc(f.b)}</h2>`);
    rows.push(row(tr("blocksTokens"), `${f.blocks} / ${f.tokens}`));
    rows.push(row(tr("graphA"), pos(f.graph_a)), row(tr("graphB"), pos(f.graph_b)));
    rows.push(row(tr("churnA"), churnStr(f.churn_a)), row(tr("churnB"), churnStr(f.churn_b)));
    rows.push(row(tr("cochange"), f.cochange === null ? tr("belowTable") : f.cochange));
  } else if (kind === "unit") {
    const u = joinDoc.units[i];
    rows.push(`<h2>${esc(u.a.path)}#${esc(u.a.key)}~${u.a.nth} ↔ ${esc(u.b.path)}#${esc(u.b.key)}~${u.b.nth}</h2>`);
    rows.push(row(tr("tokens"), u.tokens));
    rows.push(row(tr("churnA"), churnStr(u.churn_a)), row(tr("churnB"), churnStr(u.churn_b)));
    rows.push(row(tr("graphA"), u.caveat));
  } else {
    const b = dedupDoc.blocks[i];
    rows.push(`<h2>${tr("cloneBlock")}</h2>`);
    rows.push(row("a", `${b.a_file}:${b.a_start}-${b.a_end}`));
    rows.push(row("b", `${b.b_file}:${b.b_start}-${b.b_end}`));
    rows.push(row(tr("tokens"), b.tokens));
  }
  $("cand-detail").innerHTML = rows.join("");
}

candBoot();
