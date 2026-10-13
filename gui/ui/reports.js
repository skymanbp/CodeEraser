// CodeEraser GUI — the diagnostics hub (batch 4): the remaining
// report families in one screen. Every family renders its OWN
// document generically — the counts object as chips, the document's
// row arrays as tables — so this screen adds zero interpretation:
// what the CLI's --format json prints is exactly what appears.
"use strict";

// family → invoke command, whether it takes a days window, the argument name of an optional path list (paths), the dedup thresholds (thresholds), the unit-universe switch (units), the words for a scalar list the document names by key (labels), extra arguments a family reads off its own controls (args), and an optional custom renderer — a family whose document is more than flat row arrays registers one from its own file (hub_arch.js); the generic renderer stays the default.
const HUB = {
  // scan's `failed` (0.2.0): the files over a hard line, by name
  scan: { cmd: "scan_report", labels: { failed: () => tr("scanFailed") } },
  dedup: { cmd: "dedup_report", thresholds: true },
  clone: { cmd: "clone_report", units: true },
  docdup: { cmd: "docdup_report" },
  deadcode: { cmd: "deadcode_report" },
  churn: { cmd: "churn_report", days: true },
  sites: { cmd: "sites_report" },
};

// One selector option per family: the value is the family key the
// commands are registered under, the text its name in the reader's
// language.
function hubOption(name) {
  const o = document.createElement("option");
  o.value = name;
  o.textContent = tr("hubFamily", name);
  return o;
}

function registerHub(name, spec) {
  HUB[name] = spec;
  $("hub-family").appendChild(hubOption(name));
}

let hubDoc = null;

// The selector's names are the family's in the reader's language (the
// value stays the family key the commands are registered under).
function hubNames() {
  for (const o of $("hub-family").options) o.textContent = tr("hubFamily", o.value);
}

(function bootReports() {
  i18nRefreshers.push(hubNames, () => hubDoc && renderHub());
  const sel = $("hub-family");
  for (const k of Object.keys(HUB)) sel.appendChild(hubOption(k));
  sel.addEventListener("change", () => {
    $("hub-days-wrap").hidden = !HUB[sel.value].days;
    $("hub-paths-wrap").hidden = !HUB[sel.value].paths;
    $("hub-thresholds-wrap").hidden = !HUB[sel.value].thresholds;
    $("hub-units-wrap").hidden = !HUB[sel.value].units;
  });
  $("hub-load").addEventListener("click", loadHub);
})();

function loadHub() {
  const fam = HUB[$("hub-family").value];
  const args = { root: $("root").value };
  if (fam.days) args.days = posInt($("hub-days").value, 1, 14);
  if (fam.paths) args[fam.paths] = $("hub-paths").value.split(",").map((s) => s.trim()).filter(Boolean);
  // `--min-tokens` / `--min-distinct`: empty = the core's defaults
  const num = (id, lo) => ($(id).value.trim() === "" ? null : posInt($(id).value, lo, 1000000));
  if (fam.thresholds) Object.assign(args, { minTokens: num("hub-min-tokens", 1), minDistinct: num("hub-min-distinct", 0) });
  if (fam.units) args.units = $("hub-units").checked;
  if (fam.args) Object.assign(args, fam.args());
  return load("hub-load", tr("judging"), fam.cmd, args, (d) => {
    hubDoc = d;
    renderHub();
  });
}

// One chip: a name and its value, the way every hub family leads.
const hubChip = (k, v) => `<span>${esc(String(k))} <b>${esc(String(v))}</b></span>`;

// Generic document face: scalar/counts fields become chips; each
// array-of-objects field becomes a table of its first few scalar
// columns (hubTable caps it, the cap SAID).
function renderHub() {
  $("empty-reports").hidden = true;
  const fam = HUB[$("hub-family").value];
  if (fam.render) return fam.render(hubDoc);
  const chips = [];
  const tables = [];
  for (const [k, v] of Object.entries(hubDoc)) {
    if (v && typeof v === "object" && !Array.isArray(v)) {
      for (const [ck, cv] of Object.entries(v)) {
        if (typeof cv !== "object") chips.push([`${k}.${ck}`, cv]);
      }
    } else if (Array.isArray(v) && v.length && typeof v[0] === "object") {
      tables.push([k, v]);
    } else if (Array.isArray(v)) {
      // a scalar list: the names themselves are the fact, so they
      // render whole; an empty list says so
      const label = fam.labels?.[k];
      chips.push([label ? label() : k, v.length ? v.join(", ") : "—"]);
    } else if (typeof v !== "object") {
      chips.push([k, v]);
    }
  }
  $("hub-chips").innerHTML = chips.map(([k, v]) => hubChip(k, v)).join("");
  $("hub-tables").innerHTML = tables.map(hubRows).join("");
}

const HUB_ROW_CAP = 400;

// One titled table for every hub family: a heading with its count, a
// header row, the items rendered to <tr> rows — at most HUB_ROW_CAP of
// them, so a 3k-row scan stays scrollable, with the cap SAID (a
// truncated list that reads as complete would be a lie). `num(i)`
// right-aligns header cell i.
function hubTable(title, count, head, items, render = (x) => x, num = () => false) {
  const cls = (i) => (num(i) ? ' class="num"' : "");
  const cap =
    items.length > HUB_ROW_CAP ? `<div class="row zero">${esc(tr("rowsCapped", HUB_ROW_CAP, items.length))}</div>` : "";
  return (
    `<h3>${esc(String(title))} <small>${count}</small></h3>` +
    `<table><thead><tr>${head.map((h, i) => `<th${cls(i)}>${esc(h)}</th>`).join("")}</tr></thead>` +
    `<tbody>${items.slice(0, HUB_ROW_CAP).map(render).join("")}</tbody></table>` +
    cap
  );
}

// A document's array of row objects: its first five scalar columns.
function hubRows([name, rows]) {
  const cols = Object.keys(rows[0]).filter((c) => typeof rows[0][c] !== "object").slice(0, 5);
  const num = (c) => typeof rows[0][c] === "number";
  const rowOf = (r) => `<tr>${cols.map((c) => `<td${num(c) ? ' class="num"' : ""}>${esc(String(r[c] ?? ""))}</td>`).join("")}</tr>`;
  return hubTable(name, rows.length, cols, rows, rowOf, (i) => num(cols[i]));
}
