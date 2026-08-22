// CodeEraser GUI — UI-string table (M8-G3a, ruling ⑦: English is
// the default, Chinese is a lookup-table switch). PRESENTATION ONLY:
// every number and verdict still comes from the core; this file owns
// nothing but labels. Loaded first so every screen sees tr().
"use strict";

const CE_I18N = {
  en: {
    tabStructure: "structure", tabTrend: "trend", tabCandidates: "candidates",
    tabScore: "score", tabErase: "erase", tabReports: "reports",
    tabBench: "bench", tabGraph: "graph",
    benchSeries: "latency series (p50/p95 ms, self repository, release builds)",
    benchMetric: "metric", benchUnit: "cells are p50/p95 in ms; · = not measured at that version",
    benchFrozen: "frozen evaluation points (value + sealed-ledger source)",
    deep: "deep (S6)", days: "days", scan: "scan", off: "off",
    commits: "commits", load: "load",
    resolvedTo: (p) => `resolved to ${p}`,
    ratchet: "ratchet", added: "added", removed: "removed",
    overCeiling: "over ceiling", toleranceDrawn: "tolerance drawn",
    candidates: "candidates", degradedRun: "degraded — judged nothing",
    planning: "planning…", applying: "applying…",
    applied: (n) => `applied ${n} row(s) — plan converged`,
    applyConfirm: (n) => `Erase ${n} planned row(s)? A clean git worktree is required; revert is one git checkout away.`,
    eraseable: "eraseable", advisory: "advisory",
    willErase: "will erase", nothingPlanned: "nothing planned — the tree is clean",
    erasePreview: "preview plan", eraseApply: "apply plan",
    rowsCapped: (n, total) => `showing ${n} of ${total} rows`,
    emptyScore: "load to judge this repository against its committed baseline",
    emptyErase: "preview to plan what is provably safe to erase — dry-run until you apply",
    emptyReports: "pick a report family and load — the same JSON documents the CLI prints",
    emptyBench: "bench numbers load from contracts/bench/bench.json — the same sealed series the README and site quote",
    judging: "judging…", measuring: "measuring history… (minutes on a cold cache)",
    measuringMore: "measuring more…",
    joining: "joining signals… (a cold run can take minutes)",
    measureMore: (n, p) => `measure ${n} more (${p} pending)`,
    trend: "trend", window: "window", measured: "measured", pending: "pending",
    failedPrefix: "failed", windowCommits: (n) => `${n} commits`,
    date: "date", score: "score", axes: "axes", none: "none",
    depth: "depth", subdirs: "subdirs", files: "files", findings: "findings",
    deviation: "deviation",
    undeclared: "undeclared territory", declaredEmpty: "declared but empty",
    axisNames: ["geometry", "naming", "mixing", "misplaced", "docs", "stale-docs", "redundancy"],
    checkAxisNames: ["size", "complexity", "clones", "docdup", "deadcode", "churn", "cycles"],
    filePairs: "similar file pairs", unitPairs: "similar unit pairs",
    cloneBlocks: "clone blocks", selfPair: "(intra-file)",
    pairsHead: (n, d, c) => `similar file pairs — ${n} (${d}d window, ${c} commits)`,
    degraded: (why) => `graph leg degraded: ${why}`,
    blocksTokens: "blocks / tokens",
    blockTokens: (b, t) => `${b} block${b === 1 ? "" : "s"} · ${t} tokens`,
    tokensOnly: (t) => `${t} tokens`, tokens: "tokens",
    graphA: "graph a", graphB: "graph b", churnA: "churn a", churnB: "churn b",
    cochange: "co-change", belowTable: "below the report table",
    posNull: "null (unanswered)", cloneBlock: "clone block",
    entropyNames: ["naming", "dir spread"],
    divergenceOutside: "divergence: mass outside declared dirs",
    legend: "findings 0 → 4+",
    viewMap: "map", viewTree: "tree",
    split: "split advisory", splitTitle: "split-ROI advisory",
    seamAfter: "seam after", noSeam: "no seam at all — a single unit",
    cohesiveRoi: (r) => `cohesive — best seam at ROI ${r}×`,
    thDir: "directory", thOwn: "own", thSub: "Σ subtree", thFiles: "files",
    emptyStructure: "point the field above at a repository, then scan — map and tree render the same judgment",
    emptyTrend: "load to measure the score trajectory over mainline history",
    emptyCandidates: "load to join the three deletion signals over this repository",
    emptyGraph: "load to draw the file reference graph — dead files and cycles from the same judgment the CLI prints",
    graphCounts: (f, e, d, c) => `${f} files, ${e} edges — ${d} dead, ${c} cycles`, graphAlive: "alive",
    graphInOut: (i, o) => `${i} in / ${o} out`, graphCycleOf: (n) => `in a cycle of ${n} files`,
    graphUnresolved: (n) => `${n} unresolved sites — the graph refuses to know them`,
  },
  zh: {
    tabStructure: "结构", tabTrend: "趋势", tabCandidates: "删除候选", tabGraph: "引用图",
    tabScore: "分数", tabErase: "擦除", tabReports: "报告",
    tabBench: "实测",
    benchSeries: "延迟系列（p50/p95 毫秒，自仓，release 构建）",
    benchMetric: "指标", benchUnit: "单元格为 p50/p95 毫秒；· = 该版本未测",
    benchFrozen: "冻结评估点位（数值 + 封册出处）",
    deep: "深查 (S6)", days: "天数", scan: "扫描", off: "关",
    commits: "提交数", load: "加载",
    resolvedTo: (p) => `已锚定到 ${p}`,
    ratchet: "棘轮", added: "新增", removed: "移除",
    overCeiling: "超限", toleranceDrawn: "动用容差",
    candidates: "候选", degradedRun: "降级——未作判决",
    planning: "计划中…", applying: "执行中…",
    applied: (n) => `已擦除 ${n} 行——计划已收敛`,
    applyConfirm: (n) => `擦除计划中的 ${n} 行？要求干净的 git 工作区；一次 git checkout 即可回退。`,
    eraseable: "可擦", advisory: "仅建议",
    willErase: "将擦除", nothingPlanned: "无可计划——树是干净的",
    erasePreview: "预览计划", eraseApply: "执行计划",
    rowsCapped: (n, total) => `显示 ${n} / ${total} 行`,
    emptyScore: "点加载，对已提交基线判决本仓库",
    emptyErase: "点预览，计划可证安全的擦除——执行前始终是演练",
    emptyReports: "选择报告家族并加载——与 CLI 打印的同一份 JSON 文档",
    emptyBench: "实测数据来自 contracts/bench/bench.json——README 与官网引用的同一份封册系列",
    judging: "判决中…", measuring: "测量历史中…（冷缓存需数分钟）",
    measuringMore: "继续测量中…",
    joining: "联结三信号中…（冷跑可达数分钟）",
    measureMore: (n, p) => `再测 ${n} 个（余 ${p}）`,
    trend: "趋势", window: "窗口", measured: "已测", pending: "待测",
    failedPrefix: "失败", windowCommits: (n) => `${n} 个提交`,
    date: "日期", score: "分数", axes: "判轴", none: "无",
    depth: "深度", subdirs: "子目录", files: "文件", findings: "发现",
    deviation: "偏差",
    undeclared: "未声明领土", declaredEmpty: "已声明但为空",
    axisNames: ["几何", "命名", "混流", "错位", "文档", "陈旧文档", "冗余"],
    checkAxisNames: ["尺寸", "复杂度", "克隆", "文档重复", "死代码", "变动", "循环"],
    filePairs: "相似文件对", unitPairs: "相似单元对",
    cloneBlocks: "克隆块", selfPair: "（文件内部）",
    pairsHead: (n, d, c) => `相似文件对 — ${n}（${d} 天窗口，${c} 个提交）`,
    degraded: (why) => `图腿降级：${why}`,
    blocksTokens: "块数 / token 数", blockTokens: (b, t) => `${b} 块 · ${t} tokens`,
    tokensOnly: (t) => `${t} tokens`, tokens: "token 数",
    graphA: "图位 a", graphB: "图位 b", churnA: "变动 a", churnB: "变动 b",
    cochange: "共变", belowTable: "低于报告表阈值",
    posNull: "null（未作答）", cloneBlock: "克隆块",
    entropyNames: ["命名", "目录散布"],
    divergenceOutside: "散度：质量落在声明目录之外",
    legend: "发现 0 → 4+",
    viewMap: "热图", viewTree: "树状",
    split: "拆分顾问", splitTitle: "拆分 ROI 顾问",
    seamAfter: "缝在", noSeam: "根本无缝——单一单元",
    cohesiveRoi: (r) => `内聚——最优缝 ROI ${r}×`,
    thDir: "目录", thOwn: "自身", thSub: "Σ 子树", thFiles: "文件",
    emptyStructure: "在上方输入仓库路径，点扫描——热图与树状渲染同一份判决",
    emptyTrend: "点加载，测量主线历史上的分数轨迹",
    emptyCandidates: "点加载，对本仓库联结三路删除信号",
    emptyGraph: "点加载绘制文件引用图——死文件与环来自 CLI 打印的同一次判决",
    graphCounts: (f, e, d, c) => `${f} 文件，${e} 边——${d} 死，${c} 环`, graphAlive: "存活",
    graphInOut: (i, o) => `入 ${i} / 出 ${o}`, graphCycleOf: (n) => `处于 ${n} 文件环`,
    graphUnresolved: (n) => `${n} 个未解析点位——图拒绝臆测它们`,
  },
};

let ceLang = localStorage.getItem("ce-lang") === "zh" ? "zh" : "en";

function tr(key, ...args) {
  const table = CE_I18N[ceLang];
  const v = key in table ? table[key] : CE_I18N.en[key];
  return typeof v === "function" ? v(...args) : v;
}

// An axis code with no name degrades to its NUMBER rather than
// throwing. The tables cover every code the core emits today, but
// `esc(undefined)` is a TypeError, and inside toggleLang's refresher
// loop nothing catches it: the screens after the thrower would never
// re-render while ceLang and localStorage had already flipped. The
// entropy table next door already had this guard; the axis one did
// not, in four places.
function axisName(code) {
  return tr("axisNames")[code] ?? String(code);
}

// Static labels re-fill on boot and on toggle; screens re-render
// their loaded reports themselves (i18nRefreshers registered per
// screen in each file's boot).
const i18nRefreshers = [];

function applyStaticI18n() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = tr(el.dataset.i18n);
  });
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
    el.placeholder = tr(el.dataset.i18nPh);
  });
  const btn = document.getElementById("lang");
  if (btn) btn.textContent = ceLang === "en" ? "中文" : "EN";
}

function toggleLang() {
  ceLang = ceLang === "en" ? "zh" : "en";
  localStorage.setItem("ce-lang", ceLang);
  applyStaticI18n();
  i18nRefreshers.forEach((f) => f());
}
