// CodeEraser GUI — the doctor screen (K round step 6). Renders the
// ce.doctor-report document: the same object `ce doctor` prints, so
// the two faces of a DIAGNOSTIC cannot disagree about the state of
// the machine they are diagnosing. Rendering only.
//
// Two probes here deliberately differ from what SessionStart does:
// the daemon is asked without being started, and the index is peeked
// without being opened. A diagnostic that spawns a daemon, or that
// rebuilds the index it claims to be reporting on, reports a state
// it created — the backend owns that distinction, and this screen
// exists to show the result of it.
"use strict";

let doctorDoc = null;

(function bootDoctor() {
  i18nRefreshers.push(() => doctorDoc && renderDoctor());
  $("doctor-load").addEventListener("click", loadDoctor);
})();

function loadDoctor() {
  return load("doctor-load", tr("judging"), "doctor_report", { root: $("root").value }, (d) => {
    doctorDoc = d;
    renderDoctor();
  });
}

// The rows are app.js's stateRow: `bad` when the fact itself is the
// finding (no handshake, an unreadable index).

function renderDoctor() {
  $("empty-doctor").hidden = true;
  const d = doctorDoc;
  const dr = d.degradedRuns;
  const ok = d.core.handshake;
  // the handshake is the hero because it is the one fact that decides
  // whether any judgment on this machine is possible at all
  const parts = [
    `<div class="hero"><span class="hero-figure">${esc(ok ? tr("handshakeOk") : tr("handshakeFailed"))}</span>` +
    `<span class="pill ${ok ? "ok" : "bad"}">${esc(tr("handshake"))}</span></div>`,
    stateRow(tr("project"), d.root),
    stateRow(`ce`, `${d.ce.version} (proto ${d.ce.proto})`),
    ok
      ? stateRow("ce-core", `${d.core.version} (proto ${d.core.proto})`)
      : stateRow("ce-core", d.core.error ?? tr("posNull"), true),
    stateRow(tr("guardTier"), d.guard),
    // ce.doctor-report/0.4.0: every guard class the hooks read, the key
    // that sets it and its tier
    ...d.classes.map((c) => stateRow(tr("guardClass", c.class, c.key), c.tier)),
    // codes since ce.doctor-report/0.2.0 (plan v2.15) — the sentence
    // used to come off the wire in English, which is exactly the one
    // place a lookup switch cannot reach. An unknown code renders as
    // the code, never as "undefined": a state we cannot name is still
    // a state, and the number is the honest thing to show.
    stateRow(tr("indexState"), tr("indexWords", d.index.state, d.index.files), d.index.state >= 2),
    stateRow(tr("daemonState"), tr("daemonWords", d.daemon.state, d.daemon.ms), d.daemon.state === 2),
    // 0.3.0: the client deadline's residue, a row only when non-zero —
    // 0 in every healthy process
    ...(d.daemon.parkedWorkers > 0 ? [stateRow(tr("parkedWorkers"), d.daemon.parkedWorkers, true)] : []),
    // the total frames the count: the feed is append-only, so a bare
    // degraded number never returns to zero and reads as a live alarm
    stateRow(tr("degradedRuns"), tr("ofEntries", dr.degraded, dr.entries), dr.degraded > 0),
  ];
  $("doctor-body").innerHTML = parts.join("");
}
