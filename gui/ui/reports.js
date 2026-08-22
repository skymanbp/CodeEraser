// CodeEraser GUI — the diagnostics hub (batch 4): the remaining
// report families in one screen. Every family renders its OWN
// document generically — the counts object as chips, the document's
// row arrays as tables — so this screen adds zero interpretation:
// what the CLI's --format json prints is exactly what appears.
"use strict";

// family → invoke command + whether it takes a days window. The
// names are the machine vocabulary (schema ids), untranslated.
const HUB = {
  scan: { cmd: "scan_report" },
  dedup: { cmd: "dedup_report" },
  clone: { cmd: "clone_report" },
  docdup: { cmd: "docdup_report" },
  deadcode: { cmd: "deadcode_report" },
  churn: { cmd: "churn_report", days: true },
  sites: { cmd: "sites_report" },
};

let hubDoc = null;

(function bootReports() {
  i18nRefreshers.push(() => hubDoc && renderHub());
  const sel = $("hub-family");
  sel.innerHTML = Object.keys(HUB)
    .map((k) => `<option value="${k}">${k}</option>`)
    .join("");
  sel.addEventListener("change", () => {
    $("hub-days-wrap").hidden = !HUB[sel.value].days;
  });
  $("hub-load").addEventListener("click", loadHub);
})();

async function loadHub() {
  const fam = HUB[$("hub-family").value];
  $("hub-load").disabled = true;
  setStatus(tr("judging"), false);
  try {
    const args = { root: $("root").value };
    if (fam.days) args.days = posInt($("hub-days").value, 1, 14);
    hubDoc = await invoke(fam.cmd, args);
    renderHub();
    setStatus(hubDoc.schema ?? "", false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("hub-load").disabled = false;
  }
}

// Generic document face: scalar/counts fields become chips; each
// array-of-objects field becomes a table of its first few scalar
// columns, capped so a 3k-row scan stays scrollable, with the cap
// SAID (a truncated list that reads as complete would be a lie).
function renderHub() {
  $("empty-reports").hidden = true;
  const chips = [];
  const tables = [];
  for (const [k, v] of Object.entries(hubDoc)) {
    if (v && typeof v === "object" && !Array.isArray(v)) {
      for (const [ck, cv] of Object.entries(v)) {
        if (typeof cv !== "object") chips.push([`${k}.${ck}`, cv]);
      }
    } else if (Array.isArray(v) && v.length && typeof v[0] === "object") {
      tables.push([k, v]);
    } else if (typeof v !== "object") {
      chips.push([k, v]);
    }
  }
  $("hub-chips").innerHTML = chips
    .map(([k, v]) => `<span>${esc(k)} <b>${esc(String(v))}</b></span>`)
    .join("");
  $("hub-tables").innerHTML = tables.map(hubTable).join("");
}

const HUB_ROW_CAP = 400;

function hubTable([name, rows]) {
  const cols = Object.keys(rows[0]).filter((c) => typeof rows[0][c] !== "object").slice(0, 5);
  const num = (c) => (typeof rows[0][c] === "number" ? ' class="num"' : "");
  const body = rows
    .slice(0, HUB_ROW_CAP)
    .map(
      (r) =>
        `<tr>${cols.map((c) => `<td${num(c)}>${esc(String(r[c] ?? ""))}</td>`).join("")}</tr>`,
    )
    .join("");
  const capNote =
    rows.length > HUB_ROW_CAP ? `<div class="row zero">${esc(tr("rowsCapped", HUB_ROW_CAP, rows.length))}</div>` : "";
  return (
    `<h3>${esc(name)} <small>${rows.length}</small></h3>` +
    `<table><thead><tr>${cols.map((c) => `<th${num(c)}>${esc(c)}</th>`).join("")}</tr></thead>` +
    `<tbody>${body}</tbody></table>` +
    capNote
  );
}
