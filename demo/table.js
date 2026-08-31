"use strict";
// The comparison table the three READMEs embed, rendered from the three
// summaries run.js produces: the seed measured by the same six gates,
// the run without CodeEraser, and the run with it. A separate file for
// the same reason bless.js is one — run.js sits at its ratchet ceiling,
// and rendering is a second job.
//
// The table is a sequence, not a snapshot: the seed's zeros, what each
// run landed, what the Stop audit did, and — only in the run with the
// hooks — the two moves the tool itself asked for (the repair the audit
// named, then the removal `ce erase` proved safe). The gate rows are
// therefore the END of each loop: without CodeEraser nothing refuses
// anything, so that loop ends at the last write.

/** One number out of a gate's output, by the pattern its summary line carries. */
function figure(s, name, re) {
  const m = s.gates[name].out.match(re);
  return m ? m[1] : "?";
}

/** Which gate's output carries each number, and the pattern that finds it. */
const COUNTS = {
  score: ["check", /check score (\d+)\/1000/],
  blocks: ["dedup", /(\d+) clone blocks/],
  near: ["clone", /(\d+) near-miss/],
  docs: ["docdup", /(\d+) duplicate pair/],
  dead: ["deadcode", /(\d+) dead/],
  erase: ["erase", /(\d+) eraseable/],
  applied: ["applied", /erase applied: (\d+) row/],
};

/** One count out of a summary, by the name COUNTS files it under. Both
 *  shapes below need it, so it has one owner rather than a copy in each
 *  — a same-file clone js is on the size-only arm for, and therefore
 *  one no gate here could have caught. */
function count(s, key) {
  return figure(s, COUNTS[key][0], COUNTS[key][1]);
}

/** The conviction clause alone: after the `ce audit:` prefix, before the
 *  colon that opens the block list (full-width in Chinese). What follows
 *  that colon is the audit's evidence, so the cut is marked — the
 *  READMEs promise the verdict verbatim, and an elision that does not
 *  say it is one reads as the whole sentence. */
function clause(reason) {
  const body = reason.replace(/^ce audit[:：]\s*/, "");
  const head = body.split(/[:：]/)[0];
  return head === body ? head : `${head}…`;
}

/** The fail conditions `ce check` named, or null when the ratchet passed.
 *  Two runs can both end red and still not be the same red: naming them is
 *  the difference between "it failed" and "it failed on one thing less". */
function failed(s) {
  const m = s.gates.check.out.match(/failed: ([^)]+)\)/);
  return m ? m[1] : null;
}

const LABELS = {
  en: {
    seed: "The seed, by the same six gates: clone blocks · doc twins · dead files",
    landed: "Writes that landed", denied: "Denied at PreToolUse", stop: "Stop audit",
    notInLoop: "not in the loop", blocked: "**blocked** — ", mayEnd: "the turn may end",
    repaired: "The repair the audit named", auditSilent: "written, and the audit goes silent",
    erased: "`ce erase --apply`", erasedVal: (n) => `${n} row removed: the verbatim doc twin`,
    of: "of", fail: "**FAIL**", pass: "**pass**", dash: "—",
    score: "`ce check` score (ratchet)", blocks: "T1/T2 clone blocks (`ce dedup --check`, budget 0)",
    near: "near-miss clone pairs (`ce clone`)", docs: "duplicated doc segments (`ce docdup --check`)",
    dead: "dead files (`ce deadcode --check`)", erase: "provably-safe removals still planned (`ce erase --check`)",
  },
  zh: {
    seed: "种子树，同样六道门实测：克隆块 · 文档孪生 · 死文件",
    landed: "落地的写入", denied: "PreToolUse 当场拒绝", stop: "Stop 审计",
    notInLoop: "不在环内", blocked: "**拦停** — ", mayEnd: "允许结束",
    repaired: "审计点名的那处修复", auditSilent: "写下之后，审计转为沉默",
    erased: "`ce erase --apply`", erasedVal: (n) => `移除 ${n} 行：逐字文档孪生`,
    of: "/", fail: "**FAIL**", pass: "**pass**", dash: "—",
    score: "`ce check` 分数（棘轮）", blocks: "T1/T2 克隆块（`ce dedup --check`，预算 0）",
    near: "近似克隆对（`ce clone`）", docs: "重复文档段（`ce docdup --check`）",
    dead: "死文件（`ce deadcode --check`）", erase: "仍待执行的可证安全删除（`ce erase --check`）",
  },
};

/** The five numbers that carry the comparison, for the block ABOVE the
 *  table. A reader who stops here has the finding; the table below is
 *  for the one who does not. Labels stay plain prose — they render into
 *  a Markdown table and an HTML one unchanged, and one list of rows
 *  feeds both, so a homepage can never quote a figure the README
 *  lacks. */
const HEADLINE = {
  en: {
    title: "The same task, run twice",
    without: "Without CodeEraser",
    withCe: "With CodeEraser",
    refused: "writes refused before they reached disk",
    blocks: "duplicate clone blocks left behind",
    docs: "duplicated doc segments",
    erase: "removals still owed",
    score: "check score",
    cap: "One seven-step task, two identical copies of the seed; the only variable is whether CodeEraser is in the loop — the write-time guard, the Stop audit, and, once the audit refuses, the eraser acting on its own plan. Both runs still end red — not on the same things.",
  },
  zh: {
    title: "同一个任务，跑两遍",
    without: "不带 CodeEraser",
    withCe: "带 CodeEraser",
    refused: "文件落盘前被拒绝的写入",
    blocks: "残留的克隆块",
    docs: "重复文档段",
    erase: "仍欠的删除",
    score: "检查分数",
    cap: "同一个七步任务，两份完全相同的种子树；唯一的变量是 CodeEraser 在不在环内——写入时的守卫、Stop 审计，以及审计拒绝之后按自己的计划执行的擦除。两次都仍以红色收场——但红的不是同一件事。",
  },
};

/** The Markdown header row, once: the scoreboard and the twelve-row
 *  table below both print it, and the site's <th> is built from the
 *  same two strings — so no surface spells these columns its own
 *  way. */
function heading(L) {
  return `| | ${L.without} | ${L.withCe} |`;
}

/** The rows themselves, once. Both shapes below read this. */
function headlineRows(without, withCe, L) {
  return [
    [L.refused, without.denied, withCe.denied],
    [L.blocks, count(without, "blocks"), count(withCe, "blocks")],
    [L.docs, count(without, "docs"), count(withCe, "docs")],
    [L.erase, count(without, "erase"), count(withCe, "erase")],
    [L.score, count(without, "score"), count(withCe, "score")],
  ];
}

/** Markdown, for the two READMEs. The caption rides with the numbers in
 *  both shapes: five rows the with-column wins are exactly the reading a
 *  reader who stops here would carry away, and what they would carry
 *  away wrong is that either run came out clean. */
function scoreboard(without, withCe, lang) {
  const L = HEADLINE[lang];
  const rows = headlineRows(without, withCe, L).map(([k, a, b]) => `| ${k} | ${a} | **${b}** |`);
  return [heading(L), "|---|---:|---:|", ...rows, "", L.cap].join("\n") + "\n";
}

/** The same numbers as a table, for the two homepages. A comparison
 *  has two columns and the site's .install chip has one value slot,
 *  so as chips the five figures began at five different x positions,
 *  under labels that style uppercases because it was built for keys
 *  two words long. `.board` lives in site/style.css. Each figure also
 *  carries its run's name, which is what the stacked narrow-screen
 *  form reads instead of a header row it cannot show. */
function scoreboardHtml(without, withCe, lang) {
  const L = HEADLINE[lang];
  const rows = headlineRows(without, withCe, L)
    .map(([k, a, b]) => `<tr><td>${k}</td>` +
      `<td class="was" data-run="${L.without}">${a}</td>` +
      `<td class="to" data-run="${L.withCe}">${b}</td></tr>`)
    .join("\n");
  return `<h2>${L.title}</h2>\n<div class="board"><table>\n` +
    `<thead><tr><th></th><th>${L.without}</th><th>${L.withCe}</th></tr></thead>\n` +
    `<tbody>\n${rows}\n</tbody></table></div>\n<p class="cap">${L.cap}</p>\n`;
}

/** The table the three READMEs embed, from the three summaries. */
function summaryTable(seed, without, withCe, lang) {
  const L = LABELS[lang];
  const both = (f) => [f(without), f(withCe)];
  const gate = (s, name) => (s.gates[name].rc === 0 ? L.pass : L.fail);
  // the score row's verdict comes from the exit code, not from whether a
  // prose pattern matched: a check whose wording drifts still fails, and
  // reading only the prose would render that as a pass
  const verdict = (s) => (s.gates.check.rc === 0 ? L.pass : failed(s) ? `${L.fail}: ${failed(s)}` : L.fail);
  const quoted = (reason) => L.blocked + "`" + clause(reason) + "`";
  const stop = withCe.stop && withCe.stop[lang];
  const after = withCe.stopAfterRepair && withCe.stopAfterRepair[lang];
  const zeros = ["blocks", "docs", "dead"].map((k) => count(seed, k)).join(" · ");
  const rows = [
    [L.seed, zeros, zeros],
    [L.landed, ...both((s) => `${s.landed} ${L.of} ${s.landed + s.denied}`)],
    [L.denied, ...both((s) => String(s.denied))],
    [L.stop, L.notInLoop, stop ? quoted(stop) : L.mayEnd],
    [L.repaired, L.dash, after ? quoted(after) : L.auditSilent],
    [L.erased, L.dash, L.erasedVal(count(withCe, "applied"))],
    [L.score, ...both((s) => `${count(s, "score")}/1000 — ${verdict(s)}`)],
    [L.blocks, ...both((s) => `${count(s, "blocks")} (${gate(s, "dedup")})`)],
    [L.near, ...both((s) => count(s, "near"))],
    [L.docs, ...both((s) => `${count(s, "docs")} (${gate(s, "docdup")})`)],
    [L.dead, ...both((s) => `${count(s, "dead")} (${gate(s, "deadcode")})`)],
    [L.erase, ...both((s) => `${count(s, "erase")} (${gate(s, "erase")})`)],
  ];
  return [heading(HEADLINE[lang]), "|---|---|---|", ...rows.map((r) => `| ${r.join(" | ")} |`)].join("\n") + "\n";
}

module.exports = { summaryTable, scoreboard, scoreboardHtml };
