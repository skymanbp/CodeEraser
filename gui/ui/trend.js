// CodeEraser GUI — trend screen (M7-P4). Renders the
// ce.trend-report document: score points over mainline history.
// Every number shown was judged in the core; the only math here is
// pixel geometry. Uses main.js globals (invoke, $, esc, row).
"use strict";

let trendReport = null;

// Batch size for the "measure more" loop — a UI pacing choice (how
// often the progress bar refreshes), not a judgment knob.
const TREND_BATCH = 5;

function trendBoot() {
  $("trend-load").addEventListener("click", () => loadTrend(null));
  $("trend-more").addEventListener("click", () => loadTrend(TREND_BATCH));
  window.addEventListener("resize", () => trendReport && drawTrend());
}

async function loadTrend(batch) {
  const commits = Number($("trend-commits").value) || 30;
  $("trend-load").disabled = true;
  $("status").className = "";
  $("status").textContent = batch === null ? "measuring history…" : "measuring more…";
  try {
    trendReport = await invoke("trend_report", {
      root: $("root").value,
      commits,
      batch,
    });
    renderTrend();
    $("status").textContent = trendReport.schema;
  } catch (e) {
    $("status").className = "err";
    $("status").textContent = String(e);
  } finally {
    $("trend-load").disabled = false;
  }
}

function renderTrend() {
  const more = $("trend-more");
  more.hidden = trendReport.pending === 0;
  more.textContent = `measure ${Math.min(TREND_BATCH, trendReport.pending)} more (${trendReport.pending} pending)`;
  drawTrend();
  const parts = [
    `<h2>trend</h2>`,
    row("window", trendReport.window + " commits"),
    row("measured", trendReport.rows.length),
    row("pending", trendReport.pending),
  ];
  for (const [sha, why] of trendReport.failed) {
    parts.push(row("failed " + sha, esc(why)));
  }
  $("trend-detail").innerHTML = parts.join("");
}

function drawTrend() {
  const svg = $("trendchart");
  const { width, height } = svg.getBoundingClientRect();
  svg.setAttribute("viewBox", `0 0 ${width} ${height}`);
  svg.textContent = "";
  const rows = trendReport.rows;
  if (!rows.length) return;
  const ns = "http://www.w3.org/2000/svg";
  const pad = 28;
  // y = score as a fraction of its own scale — chart geometry over
  // two report facts, not a derived judgment
  const y = (r) => pad + (1 - r.score / r.scale) * (height - 2 * pad);
  const x = (i) => pad + (rows.length === 1 ? 0 : (i * (width - 2 * pad)) / (rows.length - 1));
  for (const frac of [0, 0.5, 1]) {
    const g = document.createElementNS(ns, "line");
    const gy = pad + frac * (height - 2 * pad);
    g.setAttribute("x1", pad); g.setAttribute("x2", width - pad);
    g.setAttribute("y1", gy); g.setAttribute("y2", gy);
    g.setAttribute("class", "grid");
    svg.appendChild(g);
  }
  const line = document.createElementNS(ns, "polyline");
  line.setAttribute("points", rows.map((r, i) => `${x(i)},${y(r)}`).join(" "));
  line.setAttribute("class", "trendline");
  svg.appendChild(line);
  rows.forEach((r, i) => {
    const c = document.createElementNS(ns, "circle");
    c.setAttribute("cx", x(i));
    c.setAttribute("cy", y(r));
    c.setAttribute("r", 4);
    c.setAttribute("class", "trendpt");
    const t = document.createElementNS(ns, "title");
    const when = new Date(r.ts * 1000).toISOString().slice(0, 10);
    t.textContent = `${r.commit.slice(0, 12)}  ${when}  ${r.score}/${r.scale}`;
    c.appendChild(t);
    c.addEventListener("click", () => trendPoint(r));
    svg.appendChild(c);
  });
}

function trendPoint(r) {
  $("trend-detail").innerHTML = [
    `<h2>${esc(r.commit.slice(0, 12))}</h2>`,
    row("date", new Date(r.ts * 1000).toISOString().slice(0, 19).replace("T", " ")),
    row("score", `${r.score} / ${r.scale}`),
    row("axes", r.axes.map(([c, p]) => `${c}:${p}`).join("  ") || "none"),
  ].join("");
}

trendBoot();
