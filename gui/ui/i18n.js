// CodeEraser GUI — UI-string table (M8-G3a, ruling ⑦: English is
// the default, Chinese is a lookup-table switch). PRESENTATION ONLY:
// every number and verdict still comes from the core; this file owns
// nothing but labels. Loaded first so every screen sees tr().
"use strict";

const CE_I18N = {
  en: {
    tabStructure: "structure", tabTrend: "trend", tabCandidates: "candidates",
    deep: "deep (S6)", days: "days", scan: "scan", off: "off",
    commits: "commits", load: "load",
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
    emptyStructure: "point the field above at a repository, then scan — the treemap is the judgment",
    emptyTrend: "load to measure the score trajectory over mainline history",
    emptyCandidates: "load to join the three deletion signals over this repository",
  },
  zh: {
    tabStructure: "结构", tabTrend: "趋势", tabCandidates: "删除候选",
    deep: "深查 (S6)", days: "天数", scan: "扫描", off: "关",
    commits: "提交数", load: "加载",
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
    emptyStructure: "在上方输入仓库路径，点扫描——树图就是判决",
    emptyTrend: "点加载，测量主线历史上的分数轨迹",
    emptyCandidates: "点加载，对本仓库联结三路删除信号",
  },
};

let ceLang = localStorage.getItem("ce-lang") === "zh" ? "zh" : "en";

function tr(key, ...args) {
  const table = CE_I18N[ceLang];
  const v = key in table ? table[key] : CE_I18N.en[key];
  return typeof v === "function" ? v(...args) : v;
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
