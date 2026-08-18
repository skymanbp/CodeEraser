// CodeEraser GUI — deletion-candidate browser (M7-P4). Renders the
// ce.join-report rows (three-signal file pairs + unit pairs) and the
// ce.dedup-report block list, in DOCUMENT ORDER — the list is the
// CLI's own report rows through the same report_json throats, and
// no verdict or ranking is derived here. Uses main.js globals.
"use strict";

let joinDoc = null;
let dedupDoc = null;

function candBoot() {
  $("cand-load").addEventListener("click", loadCandidates);
}

async function loadCandidates() {
  const days = Number($("cand-days").value) || 14;
  $("cand-load").disabled = true;
  $("status").className = "";
  $("status").textContent = "joining signals…";
  try {
    joinDoc = await invoke("join_report", { root: $("root").value, days });
    dedupDoc = await invoke("dedup_report", { root: $("root").value });
    renderCandidates();
    $("status").textContent = joinDoc.schema;
  } catch (e) {
    $("status").className = "err";
    $("status").textContent = String(e);
  } finally {
    $("cand-load").disabled = false;
  }
}

function renderCandidates() {
  const parts = [];
  parts.push(`<h2>similar file pairs — ${joinDoc.files.length} (${joinDoc.days}d window, ${joinDoc.commits} commits)</h2>`);
  if (joinDoc.degraded) parts.push(`<p class="err">graph leg degraded: ${esc(joinDoc.degraded)}</p>`);
  joinDoc.files.forEach((f, i) => {
    parts.push(
      `<div class="cand" data-kind="file" data-i="${i}">` +
      `<b>${esc(f.a)}</b> ↔ <b>${esc(f.b)}</b>` +
      `<span>${f.blocks} blocks · ${f.tokens} tokens</span></div>`
    );
  });
  parts.push(`<h2>similar unit pairs — ${joinDoc.units.length}</h2>`);
  joinDoc.units.forEach((u, i) => {
    parts.push(
      `<div class="cand" data-kind="unit" data-i="${i}">` +
      `<b>${esc(u.a.path)}#${esc(u.a.key)}</b> ↔ <b>${esc(u.b.path)}#${esc(u.b.key)}</b>` +
      `<span>${u.tokens} tokens</span></div>`
    );
  });
  parts.push(`<h2>clone blocks — ${dedupDoc.blocks.length}</h2>`);
  dedupDoc.blocks.forEach((b, i) => {
    parts.push(
      `<div class="cand" data-kind="block" data-i="${i}">` +
      `<b>${esc(b.a_file)}:${b.a_start}-${b.a_end}</b> ↔ ` +
      `<b>${esc(b.b_file)}:${b.b_start}-${b.b_end}</b>` +
      `<span>${b.tokens} tokens</span></div>`
    );
  });
  const list = $("cand-list");
  list.innerHTML = parts.join("");
  list.querySelectorAll(".cand").forEach((el) =>
    el.addEventListener("click", () => candDetail(el.dataset.kind, Number(el.dataset.i))));
}

const pos = (p) =>
  p === null ? "null (unanswered)" : `in ${p[0]} · out ${p[1]} · scc ${p[2]}×${p[3]} · reach ${p[4]}`;
const churnStr = (c) => `+${c.appended} / ~${c.rewrote}`;

function candDetail(kind, i) {
  const rows = [];
  if (kind === "file") {
    const f = joinDoc.files[i];
    rows.push(`<h2>${esc(f.a)} ↔ ${esc(f.b)}</h2>`);
    rows.push(row("blocks / tokens", `${f.blocks} / ${f.tokens}`));
    rows.push(row("graph a", pos(f.graph_a)), row("graph b", pos(f.graph_b)));
    rows.push(row("churn a", churnStr(f.churn_a)), row("churn b", churnStr(f.churn_b)));
    rows.push(row("co-change", f.cochange === null ? "below the report table" : f.cochange));
  } else if (kind === "unit") {
    const u = joinDoc.units[i];
    rows.push(`<h2>${esc(u.a.path)}#${esc(u.a.key)}~${u.a.nth} ↔ ${esc(u.b.path)}#${esc(u.b.key)}~${u.b.nth}</h2>`);
    rows.push(row("tokens", u.tokens));
    rows.push(row("churn a", churnStr(u.churn_a)), row("churn b", churnStr(u.churn_b)));
    rows.push(row("graph", esc(u.caveat)));
  } else {
    const b = dedupDoc.blocks[i];
    rows.push(`<h2>clone block</h2>`);
    rows.push(row("a", `${esc(b.a_file)}:${b.a_start}-${b.a_end}`));
    rows.push(row("b", `${esc(b.b_file)}:${b.b_start}-${b.b_end}`));
    rows.push(row("tokens", b.tokens));
  }
  $("cand-detail").innerHTML = rows.join("");
}

candBoot();
