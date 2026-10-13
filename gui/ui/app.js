// CodeEraser GUI shell (batch 4 split of main.js at the repo's own
// soft line): the shared globals every screen uses (`invoke`, `$`,
// `esc`, `row`, `setStatus`, `posInt`, `redrawers`, the alarm ramp),
// the tab strip, root memory + the anchoring echo, the ce-task
// event feed, boot, and the helpers more than one screen reads
// (`load`, `stateRow`, `census`). Screens own their rendering; this file owns
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

// The OS folder picker beside the root field (tauri-plugin-dialog in
// directory mode): a second way to fill the SAME field. The picked
// path goes through the field's own change handler, so it is
// remembered and echoed exactly as a typed one; a cancelled dialog
// answers null and changes nothing.
async function pickRoot() {
  try {
    const dir = await window.__TAURI__.dialog.open({
      directory: true, title: tr("pickRoot"), defaultPath: $("root").value || undefined,
    });
    if (!dir) return;
    $("root").value = dir;
    $("root").dispatchEvent(new Event("change"));
  } catch (e) {
    setStatus(String(e), true);
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
  $("pick").addEventListener("click", pickRoot);
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

// A row whose value carries state (the doctor and update screens):
// `bad` when the fact itself is the finding, quiet otherwise. The words
// are the backend's — no screen mints a diagnosis.
const stateRow = (k, v, bad) =>
  `<div class="row${bad ? " bad" : ""}"><span>${esc(k)}</span>${esc(String(v))}</div>`;

// The one load road every screen takes: its button(s) disabled while
// the backend works, a status word meanwhile (`said`, translated at the
// call so the i18n gate sees its key), then the document's
// schema (or what `show` returns instead) — or the error, whole, on the
// status line. `show` stores and renders the document; the document is
// returned (undefined on error).
async function load(btns, said, cmd, args, show) {
  const ids = [].concat(btns);
  ids.forEach((id) => ($(id).disabled = true));
  setStatus(said, false);
  try {
    const d = await invoke(cmd, args);
    const status = show(d);
    setStatus(status ?? d.schema ?? "", false);
    return d;
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    ids.forEach((id) => ($(id).disabled = false));
  }
}

// How many items landed on each key, ordered by the core's severity
// (higher first; `sevOf` read off the first item of a key), ties by the
// key itself. Counting is not judging: every key and every rank came off
// the wire. Returns [key, count] pairs.
function census(items, keyOf, sevOf) {
  const tally = new Map();
  for (const it of items) {
    const k = keyOf(it);
    if (k == null) continue;
    const cur = tally.get(k) ?? { n: 0, sev: sevOf(it) ?? 0 };
    cur.n += 1;
    tally.set(k, cur);
  }
  const byKey = (a, b) => (typeof a === "number" ? a - b : String(a).localeCompare(String(b)));
  return [...tally.entries()].sort((a, b) => b[1].sev - a[1].sev || byKey(a[0], b[0])).map(([k, v]) => [k, v.n]);
}
