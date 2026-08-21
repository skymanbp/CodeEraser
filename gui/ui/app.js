// CodeEraser GUI shell (batch 4 split of main.js at the repo's own
// soft line): the shared globals every screen uses (`invoke`, `$`,
// `esc`, `row`, `setStatus`, `posInt`, `redrawers`, the alarm ramp),
// the tab strip, root memory + the anchoring echo, the ce-task
// event feed, and boot. Screens own their rendering; this file owns
// the chrome. No framework, no build step — judgment zero-leak into
// JS (user ruling).
"use strict";

const invoke = window.__TAURI__.core.invoke;
const $ = (id) => document.getElementById(id);

// Per-screen redraw hooks: switching back to a tab re-renders its
// SVG, because a chart drawn (or resized) while its view was hidden
// measured a 0×0 rect and holds no usable geometry.
const redrawers = {};

// The alarm ramp: 0 findings = calm panel green-grey; the four hot
// stops sit on even lightness steps so adjacent counts stay
// separable (capped at 4 — beyond that it is simply the hot end).
const LEGEND_STOPS = ["#222932", "#61332a", "#93402f", "#c2503d", "#e8705a"];
function heat(findings) {
  return LEGEND_STOPS[Math.min(findings, LEGEND_STOPS.length - 1)];
}

function tabs() {
  const all = document.querySelectorAll("#tabs .tab");
  all.forEach((b) =>
    b.addEventListener("click", () => {
      all.forEach((x) => x.classList.toggle("on", x === b));
      document.querySelectorAll(".view").forEach((v) => {
        v.hidden = v.id !== "view-" + b.dataset.tab;
      });
      redrawers[b.dataset.tab]?.();
    }));
}

// One status line for every screen; long errors ellipsize in CSS, so
// the full text rides on the tooltip.
function setStatus(text, isErr) {
  const s = $("status");
  s.className = isErr ? "err" : "";
  s.textContent = text;
  s.title = isErr ? text : "";
}

// Number inputs guard here, not just in the HTML min attribute —
// typed garbage or a negative would otherwise reach the backend as a
// deserialization error. Invalid → fallback (null = axis unjudged).
function posInt(value, min, fallback) {
  const n = Math.floor(Number(value));
  return Number.isFinite(n) && n >= min ? n : fallback;
}

// The typed root anchors backend-side (crate::root); this echo shows
// WHERE it landed the moment the field settles, so a subdirectory
// user learns their judgment covers the whole project, not the sub.
async function echoRoot() {
  try {
    const r = await invoke("resolve_root", { root: $("root").value });
    $("rootecho").textContent = r.ascended ? tr("resolvedTo", r.root) : "";
    $("rootecho").title = r.root;
  } catch {
    $("rootecho").textContent = "";
  }
}

// ce-task progress feed: every backend command brackets itself with
// start/done/error events — long judgments tick in the status line
// instead of freezing it.
function listenTasks() {
  const ev = window.__TAURI__.event;
  ev.listen("ce-task", ({ payload }) => {
    if (payload.state === "start") setStatus(`${payload.command}…`, false);
  }).catch(() => {});
}

async function boot() {
  applyStaticI18n();
  $("lang").addEventListener("click", toggleLang);
  tabs();
  listenTasks();
  const remembered = localStorage.getItem("ce-root");
  $("root").value = remembered || (await invoke("default_root").catch(() => ""));
  $("root").addEventListener("change", () => {
    localStorage.setItem("ce-root", $("root").value);
    echoRoot();
  });
  echoRoot();
  bootStructure();
}

// Deferred scripts all execute before DOMContentLoaded, so by the
// time boot runs every screen's functions exist regardless of file
// order — the split must never depend on which file loads last.
document.addEventListener("DOMContentLoaded", boot);

// BOTH sides escaped. The key is an i18n literal at almost every call
// site, but not all of them — trend.js builds one from report data —
// and an asymmetry that holds only by convention is one call site away
// from being false.
const row = (k, v) => `<div class="row">${esc(String(k))}: <b>${esc(String(v))}</b></div>`;
const esc = (s) =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
