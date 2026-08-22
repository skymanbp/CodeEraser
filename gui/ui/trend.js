// CodeEraser GUI — trend screen (M7-P4, i18n'd M8-G3a). Renders the
// ce.trend-report document: score points over mainline history plus
// the core's trend/1 judgment. Every number shown was judged in the
// core; the only math here is pixel geometry. Labels via i18n.js.
"use strict";

let trendReport = null;

// Batch size for the "measure more" loop — a UI pacing choice (how
// often the progress bar refreshes), not a judgment knob.
const TREND_BATCH = 5;

function trendBoot() {
  $("trend-load").addEventListener("click", () => loadTrend(null));
  $("trend-more").addEventListener("click", () => loadTrend(TREND_BATCH));
  window.addEventListener("resize", () => trendReport && drawTrend());
  redrawers.trend = () => trendReport && drawTrend();
  i18nRefreshers.push(() => trendReport && renderTrend());
}

async function loadTrend(batch) {
  const commits = posInt($("trend-commits").value, 2, 30);
  $("trend-load").disabled = true;
  $("trend-more").disabled = true;
  setStatus(batch === null ? tr("measuring") : tr("measuringMore"), false);
  try {
    // Load (batch=null) measures in TREND_BATCH slices, rendering
    // after each — the old monolithic call froze for minutes
    // (v0.7.3). A slice that shrinks nothing stops the walk: failed
    // commits never leave pending, and renderTrend lists them.
    let prev = Infinity;
    do {
      trendReport = await invoke("trend_report", {
        root: $("root").value,
        commits,
        batch: batch === null ? TREND_BATCH : batch,
      });
      renderTrend();
      setStatus(`${tr("measured")} ${trendReport.rows.length} · ${tr("pending")} ${trendReport.pending}`, false);
      if (trendReport.pending >= prev) break;
      prev = trendReport.pending;
    } while (batch === null && trendReport.pending > 0);
    setStatus(trendReport.schema, false);
  } catch (e) {
    setStatus(String(e), true);
  } finally {
    $("trend-load").disabled = false;
    $("trend-more").disabled = false;
  }
}

function renderTrend() {
  $("empty-trend").hidden = true;
  const more = $("trend-more");
  more.hidden = trendReport.pending === 0;
  more.textContent = tr("measureMore", Math.min(TREND_BATCH, trendReport.pending), trendReport.pending);
  drawTrend();
  const rows = trendReport.rows;
  const parts = [`<h2>${tr("trend")}</h2>`];
  if (rows.length) {
    // the one number this screen is about leads the aside
    const last = rows[rows.length - 1];
    parts.push(`<div class="bigscore">${last.score}<small> / ${last.scale}</small></div>`);
  }
  parts.push(
    row(tr("window"), tr("windowCommits", trendReport.window)),
    row(tr("measured"), rows.length),
    row(tr("pending"), trendReport.pending),
  );
  for (const [sha, why] of trendReport.failed) {
    parts.push(row(`${tr("failedPrefix")} ${sha}`, why));
  }
  $("trend-detail").innerHTML = parts.join("");
}

// The y-domain is data-driven with a 2%-of-scale span floor and 35%
// headroom, clamped to [0, scale] — a flat 820-ish series must not
// compress into a 2px band on a 0..1000 canvas. The non-zero baseline
// is DECLARED: ticks carry the real values and the endpoint label
// carries "/ scale".
function trendDomain(rows, scale) {
  const scores = rows.map((r) => r.score);
  const lo0 = Math.min(...scores);
  const hi0 = Math.max(...scores);
  const span = Math.max(hi0 - lo0, Math.round(scale * 0.02));
  const lo = Math.max(0, lo0 - Math.round(span * 0.35));
  const hi = Math.min(scale, hi0 + Math.round(span * 0.35));
  // A FLAT series under a small scale collapses the domain to a point:
  // the 2% span floor rounds to 0 below scale 25 (score_scale is a
  // ce.toml knob validated only as >= 1), and y() then divides by zero
  // into NaN coordinates — a silently blank chart, no error anywhere.
  // The x() twin right below guards its own single-point case.
  return hi > lo ? { lo, hi } : { lo, hi: lo + 1 };
}

function drawTrend() {
  const svg = $("trendchart");
  const { width, height } = svg.getBoundingClientRect();
  if (!width || !height) return; // hidden view measures 0×0 — keep the old drawing
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.textContent = "";
  const rows = trendReport.rows;
  if (!rows.length) return;
  const ns = "http://www.w3.org/2000/svg";
  // padL leaves room for four tabular glyphs of tick; gridlines snap
  // to the half-pixel grid so a 1px stroke does not antialias into
  // two blurred rows
  const padL = 56;
  const pad = 34;
  const scale = rows[0].scale; // the report's own score scale
  const { lo, hi } = trendDomain(rows, scale);
  const y = (r) => pad + (1 - (r.score - lo) / (hi - lo)) * (height - 2 * pad);
  const x = (i) => padL + (rows.length === 1 ? 0 : (i * (width - padL - pad)) / (rows.length - 1));
  for (const frac of [0, 0.5, 1]) {
    const g = document.createElementNS(ns, "line");
    const gy = Math.round(pad + frac * (height - 2 * pad)) + 0.5;
    g.setAttribute("x1", padL); g.setAttribute("x2", width - pad);
    g.setAttribute("y1", gy); g.setAttribute("y2", gy);
    g.setAttribute("class", "grid");
    svg.appendChild(g);
    const tick = document.createElementNS(ns, "text");
    tick.setAttribute("x", padL - 10);
    tick.setAttribute("y", gy + 4);
    tick.setAttribute("class", "axis tick");
    tick.textContent = Math.round(hi - frac * (hi - lo));
    svg.appendChild(tick);
  }
  // x extent: first/last commit + date — index-spaced, so the honest
  // axis names the endpoints rather than faking a time scale
  const xcap = (r, anchor, px) => {
    const t = document.createElementNS(ns, "text");
    t.setAttribute("x", px);
    t.setAttribute("y", height - pad + 16);
    t.setAttribute("class", "axis");
    t.setAttribute("text-anchor", anchor);
    t.textContent = `${new Date(r.ts * 1000).toISOString().slice(0, 10)} ${r.commit.slice(0, 7)}`;
    return t;
  };
  svg.appendChild(xcap(rows[0], "start", padL));
  if (rows.length > 1) svg.appendChild(xcap(rows[rows.length - 1], "end", width - pad));
  const line = document.createElementNS(ns, "polyline");
  line.setAttribute("points", rows.map((r, i) => `${x(i)},${y(r)}`).join(" "));
  line.setAttribute("class", "trendline");
  svg.appendChild(line);
  rows.forEach((r, i) => {
    const c = document.createElementNS(ns, "circle");
    c.setAttribute("cx", x(i));
    c.setAttribute("cy", y(r));
    c.setAttribute("r", 4);
    c.setAttribute("class", i === rows.length - 1 ? "trendpt end" : "trendpt");
    const t = document.createElementNS(ns, "title");
    const when = new Date(r.ts * 1000).toISOString().slice(0, 10);
    t.textContent = `${r.commit.slice(0, 12)}  ${when}  ${r.score}/${r.scale}`;
    c.appendChild(t);
    c.addEventListener("click", () => trendPoint(r));
    svg.appendChild(c);
  });
  const last = rows[rows.length - 1];
  const lbl = document.createElementNS(ns, "text");
  const lx = x(rows.length - 1);
  lbl.setAttribute("x", Math.min(lx + 8, width - pad));
  lbl.setAttribute("y", y(last) - 10);
  lbl.setAttribute("class", "endlabel");
  if (lx > width - 110) lbl.setAttribute("text-anchor", "end");
  lbl.textContent = `${last.score} / ${last.scale}`;
  svg.appendChild(lbl);
}

function trendPoint(r) {
  $("trend-detail").innerHTML = [
    `<h2>${esc(r.commit.slice(0, 12))}</h2>`,
    row(tr("date"), new Date(r.ts * 1000).toISOString().slice(0, 19).replace("T", " ")),
    row(tr("score"), `${r.score} / ${r.scale}`),
    row(tr("axes"), r.axes.map(([c, p]) => `${axisName(c)}:${p}`).join("  ") || tr("none")),
  ].join("");
}

trendBoot();
