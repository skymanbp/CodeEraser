// CodeEraser GUI — the update screen. Renders the ce.update-report
// document: the same object `ce update` prints, so the GUI and the
// CLI cannot disagree about which release is latest or what this
// install may do about it. Rendering only: the check, the pin
// verification and the placement all live in the library
// (codeeraser::update), and the "update now" button is the one
// destructive act on this screen — confirmed, like erase apply.
"use strict";

let updateDoc = null;

(function bootUpdate() {
  i18nRefreshers.push(() => updateDoc && renderUpdate());
  $("update-check").addEventListener("click", loadUpdate);
  $("update-apply").addEventListener("click", applyUpdate);
})();

function loadUpdate() {
  return load("update-check", tr("checking"), "update_check", {}, (d) => {
    updateDoc = d;
    renderUpdate();
  });
}

async function applyUpdate() {
  const d = updateDoc;
  if (!d || !window.confirm(tr("updateConfirm", d.latest.version))) return;
  $("update-apply").disabled = true;
  setStatus(tr("updating"), false);
  try {
    const done = await invoke("update_apply", { installer: $("update-installer").checked });
    const lines = [tr("updatePlaced", done.version, done.placed.join(", "))];
    if (done.installer) lines.push(tr("installerSaved", done.installer));
    $("update-body").insertAdjacentHTML("beforeend", lines.map((l) => `<div class="row ok"><span></span>${esc(l)}</div>`).join(""));
    setStatus(lines[0], false);
    $("update-apply").hidden = true;
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("update-apply").disabled = false;
  }
}

// Every apply-side act is gated on the document's own action code —
// the button appears only where the library would place anything,
// and the installer box only where a bundle sits beside us.
function renderUpdate() {
  $("empty-update").hidden = true;
  const d = updateDoc;
  const cur = d.current;
  const late = d.latest;
  const parts = [
    `<div class="hero"><span class="hero-figure">${esc(cur.version)}</span>` +
    `<span class="pill ${d.verdict === 0 ? "ok" : "bad"}">${esc(tr("updateVerdictWords", d.verdict))}</span></div>`,
    stateRow(tr("currentVersion"), `${cur.version} (proto ${cur.proto}) — ${cur.exe}`),
    stateRow(tr("installKind"), tr("installWords", cur.install)),
    late.error
      ? stateRow(tr("latestVersion"), late.error, true)
      : stateRow(tr("latestVersion"), `${late.version} — ${late.url}`, d.verdict === 1),
    ...(d.pins.error ? [stateRow(tr("updatePins"), d.pins.error, true)] : []),
    ...(d.pins.ce ? [stateRow(tr("updatePins"), `ce ${d.pins.ce.slice(0, 12)}… · ce-core ${d.pins.ceCore.slice(0, 12)}…`)] : []),
    ...(d.verdict === 1 ? [stateRow(tr("verdict"), tr("updateActionWords", d.action), d.action === 2 || d.action === 3)] : []),
  ];
  $("update-body").innerHTML = parts.join("");
  const here = d.verdict === 1 && (d.action === 1 || d.action === 4);
  $("update-apply").hidden = !here;
  $("update-installer-wrap").hidden = !(here && d.pins.installer);
  $("update-installer").checked = d.action === 4;
}
