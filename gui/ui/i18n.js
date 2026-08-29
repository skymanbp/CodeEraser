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
    dropped: "dropped (on disk, unmeasured)", scanFailed: "failed",
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
    // the unit tier's graph leg and WHY it is null, keyed by the
    // wire's caveat code (ce.join-report/0.3.0) — the sentence used
    // to ride the wire in English, where no switch could reach it.
    // Since 6.2.0 the symbol level has a face of its own (the
    // deadcode advisory on the graph screen); the join's graph leg is
    // still file-tier, so the unit row stays null and says why.
    graphNull: "graph", graphNullWhy: {
      1: "null — the graph judges files, not units; a unit's symbol-level reading is the deadcode advisory on the graph screen, not an indegree",
    },
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
    emptyGraph: "load to draw the file reference graph — dead files and cycles from the canvas judgment, the unmentioned-declaration advisory from the deadcode report the CLI prints",
    graphCounts: (f, e, d, c) => `${f} files, ${e} edges — ${d} dead, ${c} cycles`, graphAlive: "alive",
    graphInOut: (i, o) => `${i} in / ${o} out`, graphCycleOf: (n) => `in a cycle of ${n} files`,
    graphUnresolved: (n) => `${n} unresolved sites — the graph refuses to know them`,
    // the symbol-level advisory (ce.deadcode-report 0.3.0, plan v2.17
    // L round): declarations no other file spells, by the core's code
    advisoryHead: (n, f) => `${n} unmentioned declaration${n === 1 ? "" : "s"} in ${f} file${f === 1 ? "" : "s"} — advisory, never a verdict`,
    advisoryHover: (n) => `${n} unmentioned`,
    advisoryWords: {
      public_unmentioned: "public — exported, spelled by no other file",
      private_unmentioned: "private — reachable only inside its own package",
      restricted_unmentioned: "restricted — visible to its crate alone",
      reexported_unmentioned: "re-exported — reachable only through a façade",
    },
    advisoryDropped: "the core dropped the unmentioned table — over its row cap, nothing judged at symbol level",
    advisoryCut: "the candidate table was cut at the producer's row cap — these rows are a prefix, the same prefix every run",
    advisoryUnavailable: "the advisory road failed — the map is drawn, no symbol-level reading is shown",
    // Judgment vocabulary (K round step 6). Every one of these labels
    // names a number the core produced and this screen used to drop:
    // the trend judgment, the join lattice's verdict/severity/legs,
    // and the graph family's trust column.
    verdict: "verdict", severity: "severity", legsAgree: "legs agreeing",
    trendVerdictNames: ["improving", "flat", "degrading"],
    unjudged: "unjudged — below minPoints",
    slopePerDay: (p) => `${p}‰ / day`,
    steepestDrop: "steepest single step", declineRun: "longest decline run",
    dropAt: (d, i) => `−${d}‰ at point ${i}`,
    runFrom: (i, n) => `${n} points from point ${i}`,
    floorArmed: (f) => `fail under ${f}`, floorOff: "ratchet only — no floor armed",
    floorHint: "score floor (blank = ratchet only, as the CLI defaults)",
    joinVerdictNames: ["report_only", "merge", "delete", "churn_hotspot"],
    trustNames: ["unvouched — unresolved sites in this language", "vacuous", "vouched"],
    trust: "trust", byVerdict: "by verdict",
    tabDoctor: "doctor", handshake: "handshake", project: "project",
    guardTier: "guard tier", indexState: "index", daemonState: "daemon",
    // the doctor document carries CODES since ce.doctor-report/0.2.0
    // (plan v2.15); an unknown code shows AS the code, because a state
    // this table cannot name is still a state the reader should see
    indexWords: (s, n) => [
      "absent (first dedup/probe builds it)",
      `${n} files`,
      `${n} files (stale — next dedup rebuilds it)`,
      "unreadable (degraded — deep checks off until rebuilt)",
    ][s] ?? `state ${s}`,
    daemonWords: (s, ms) => [
      `warm (${ms} ms)`,
      "not running (lazy-starts on first probe)",
      "unreachable (DEGRADED: cheap checks only, guard fails open)",
    ][s] ?? `state ${s}`,
    degradedRuns: "degraded runs", ofEntries: (n, t) => `${n} of ${t} feed entries`,
    parkedWorkers: "parked daemon workers (past the client deadline, not returned)",
    emptyDoctor: "load to read this machine's state — the same document `ce doctor` prints, and the daemon is asked without being started",
    // the update document carries CODES like the doctor's (plan
    // v2.15): install kind, verdict and action each render from a
    // table here, and an unknown code shows AS the code
    tabUpdate: "update", updateCheck: "check for updates", updateApply: "update now",
    updateInstaller: "also save the verified GUI installer",
    emptyUpdate: "check to compare this build against the latest release — the same document `ce update` prints; nothing is downloaded until you apply",
    checking: "checking…", updating: "downloading and verifying…",
    currentVersion: "this build", latestVersion: "latest release", installKind: "install",
    updatePins: "pins (SHA-256, from the release commit)",
    installWords: (s) => ["placed by hand", "installer bundle sidecar", "cargo install", "plugin starter's bound copy"][s] ?? `install ${s}`,
    updateVerdictWords: (s) => ["up to date", "update available", "unknown"][s] ?? `verdict ${s}`,
    updateActionWords: (s) => [
      "nothing to do",
      "update now replaces ce and ce-core in place (pins verified)",
      "this copy is the plugin's: run /plugin update codeeraser in Claude Code",
      "this copy is cargo's: run cargo install codeeraser",
      "update now replaces ce and ce-core beside the app; tick the installer box to save the verified installer for the GUI app itself",
    ][s] ?? `action ${s}`,
    updateConfirm: (v) => `Update to ${v}? ce and ce-core are replaced in place after their SHA-256 pins verify; the previous copies are retired as .old.`,
    updatePlaced: (v, list) => `updated to ${v}: placed ${list}`,
    installerSaved: (p) => `installer saved (verified): ${p} — run it to update the GUI app`,
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
    dropped: "掉线（在盘未量）", scanFailed: "失败条件",
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
    graphNull: "图位", graphNullWhy: {
      1: "null——图判的是文件不是单元；单元的符号层读数是引用图屏的死码顾问，不是入度",
    },
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
    emptyGraph: "点加载绘制文件引用图——死文件与环来自画布判决，未提及声明顾问来自 CLI 打印的 deadcode 报告",
    graphCounts: (f, e, d, c) => `${f} 文件，${e} 边——${d} 死，${c} 环`, graphAlive: "存活",
    graphInOut: (i, o) => `入 ${i} / 出 ${o}`, graphCycleOf: (n) => `处于 ${n} 文件环`,
    graphUnresolved: (n) => `${n} 个未解析点位——图拒绝臆测它们`,
    advisoryHead: (n, f) => `${n} 个未提及声明分布于 ${f} 个文件——仅建议，永不判决`,
    advisoryHover: (n) => `${n} 个未提及`,
    advisoryWords: {
      public_unmentioned: "公开——已导出，无他文件拼写其名",
      private_unmentioned: "私有——只在自身包内可达",
      restricted_unmentioned: "受限——仅对本 crate 可见",
      reexported_unmentioned: "再导出——只经门面可达",
    },
    advisoryDropped: "核已丢弃未提及表——超出行上限，符号层一行未判",
    advisoryCut: "候选表已在生产者侧行上限截断——以上各行是前缀，每次运行同一前缀",
    advisoryUnavailable: "顾问路失败——图已绘出，不显示符号层读数",
    verdict: "判决", severity: "严重度", legsAgree: "佐证腿数",
    trendVerdictNames: ["上行", "持平", "恶化"],
    unjudged: "未判——低于最小点数",
    slopePerDay: (p) => `${p}‰ / 日`,
    steepestDrop: "最陡单步", declineRun: "最长连跌",
    dropAt: (d, i) => `第 ${i} 点跌 ${d}‰`,
    runFrom: (i, n) => `自第 ${i} 点起连跌 ${n} 点`,
    floorArmed: (f) => `低于 ${f} 即判失败`, floorOff: "仅棘轮——未武装地板",
    floorHint: "分数地板（留空 = 仅棘轮，与 CLI 默认一致）",
    joinVerdictNames: ["report_only", "merge", "delete", "churn_hotspot"],
    trustNames: ["未担保——该语言尚有未解析点位", "空担保", "已担保"],
    trust: "担保", byVerdict: "按判决",
    tabDoctor: "体检", handshake: "握手", project: "项目",
    guardTier: "守卫档位", indexState: "索引", daemonState: "daemon",
    indexWords: (s, n) => [
      "缺失（首次 dedup/probe 会建立）",
      `${n} 个文件`,
      `${n} 个文件（陈旧 — 下次 dedup 重建）`,
      "不可读（已降级 — 重建前深检关闭）",
    ][s] ?? `状态 ${s}`,
    daemonWords: (s, ms) => [
      `已预热（${ms} 毫秒）`,
      "未运行（首次 probe 时惰性启动）",
      "不可达（已降级：仅剩廉价检查，守卫失败开放）",
    ][s] ?? `状态 ${s}`,
    degradedRuns: "降级运行", ofEntries: (n, t) => `${n} / ${t} 条流水`,
    parkedWorkers: "滞留的 daemon 工人线程（客户端期限已过仍未返回）",
    emptyDoctor: "点加载读取本机状态——与 `ce doctor` 打印的是同一份文档，且探 daemon 不启动它",
    tabUpdate: "更新", updateCheck: "检查更新", updateApply: "立即更新",
    updateInstaller: "同时保存已校验的 GUI 安装包",
    emptyUpdate: "点检查把本构建与最新发布对比——与 `ce update` 打印的是同一份文档；应用前不下载任何东西",
    checking: "检查中…", updating: "下载并校验中…",
    currentVersion: "本构建", latestVersion: "最新发布", installKind: "安装方式",
    updatePins: "pin（SHA-256，来自发布提交）",
    installWords: (s) => ["手工放置", "安装包随附", "cargo 安装", "插件启动器绑定副本"][s] ?? `安装方式 ${s}`,
    updateVerdictWords: (s) => ["已是最新", "有更新", "未知"][s] ?? `判定 ${s}`,
    updateActionWords: (s) => [
      "无需操作",
      "立即更新会就地替换 ce 与 ce-core（按 pin 校验）",
      "此副本属于插件：在 Claude Code 里运行 /plugin update codeeraser",
      "此副本由 cargo 管理：运行 cargo install codeeraser",
      "立即更新会在应用旁替换 ce 与 ce-core；勾选安装包即为 GUI 应用本体保存已校验的安装包",
    ][s] ?? `操作 ${s}`,
    updateConfirm: (v) => `更新到 ${v}？ce 与 ce-core 在 SHA-256 pin 校验通过后就地替换；旧副本改名为 .old 退役。`,
    updatePlaced: (v, list) => `已更新到 ${v}：已放置 ${list}`,
    installerSaved: (p) => `安装包已保存（已校验）：${p} — 运行它以更新 GUI 应用`,
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
  // A tooltip is prose like any other: an untranslated `title` is the
  // same leak the placeholder arm exists to prevent, and the i18n
  // gate harvests this attribute for the same reason.
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    el.title = tr(el.dataset.i18nTitle);
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
