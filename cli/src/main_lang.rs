//! Runtime help localization (M8-G3b): under zh the built Command's
//! about/help strings are swapped from the lookup below — the
//! English surface stays the derive's own doc comments (zero
//! normalized tokens by construction). The zh side travels as ONE
//! tab-separated literal: the tuple-table first cut T2-matched two
//! unrelated data tables in the corpus (hs_boot's module rows,
//! notice_gate's census rows) — under literal abstraction any
//! homogeneous tuple list matches any other, so the data is one
//! token instead. The completeness test at the bottom is the
//! charter's G3 acceptance: a missing zh entry is red, a dead key
//! is red. clap's own framework vocabulary (error:, Usage:, Print
//! help) stays English — the FAIL/pass stance applied to the parser.

use std::collections::HashMap;
use std::sync::OnceLock;

/// `key<TAB>zh` per line. Keys: `ce` = root about, `ce.lang` = the
/// global flag, `<cmd>` = a subcommand's about, `<cmd>.<arg id>` =
/// an arg's help, and `judge.<arg id>` = the shared JudgeArgs flag
/// set's ONE authority, consulted when the command names no key of
/// its own (see zh_help).
const ZH_TSV: &str = "\
ce	CodeEraser — 消除 LLM 引入的代码与文档熵
ce.lang	控制台语言（优先于 CE_LANG）
judge.root	要分析的目录（默认当前目录）
judge.core	ce-core 可执行文件路径（默认：CE_CORE_BIN、与本二进制同目录的 ce-core、再 PATH）
judge.db	索引数据库路径（默认 <root>/.ce/index.db）
doctor	环境与项目健康：ce-core 握手、项目状态行、降级计数（绝不启动守护进程）
doctor.core	ce-core 可执行文件路径（默认：CE_CORE_BIN、与本二进制同目录的 ce-core、再 PATH）
doctor.root	要报告的项目根（默认当前目录）
scan	度量尺寸 / 复杂度 / 可读性指标；级别由核分级
scan.path	要扫描的目录（默认当前目录）
scan.core	ce-core 可执行文件路径（默认：CE_CORE_BIN、与本二进制同目录的 ce-core、再 PATH）
churn	时间维度指标：追加对重写、窗口改动、共变对（仅报告；联判消费之）。默认窗口需数分钟——逐提交一个 git 子进程、逐触及文件一次 blame；进度走 stderr
churn.root	仓库根（默认当前目录）
churn.days	历史窗口天数
graph	依赖图子系统：--sites 列出引用站点（不做解析）；--mentions 刷新提及语料宇宙并报其头部；存活性判决在 `ce deadcode`
graph.root	要分析的目录（默认当前目录）
graph.sites	列出引用站点
graph.mentions	刷新提及语料宇宙（树中每个可能引用到名字的文本文件）并报告其内容
deadcode	在缓存引用图上判决存活性：阶梯的边，核的四路判决，以及符号层顾问（无他文件拼写其名的声明——永不判决）
deadcode.root	要判决的目录（默认当前目录）
deadcode.db	索引数据库路径（默认 <root>/.ce/index.db）
deadcode.core	ce-core 可执行文件路径（默认：CE_CORE_BIN、与本二进制同目录的 ce-core、再 PATH）
deadcode.check	任一文件级死判落地即退出 1
clone	T3 近似克隆判决：经核 clone/1 的树编辑距离；--units 改为列出缓存单元宇宙
clone.units	改为列出单元宇宙而非判决
docdup	文档重复判决：对缓存活段经核 docdup/1 精确 Jaccard
docdup.check	任一重复被报告即退出 1（CI 自食门）
join	三信号联判：相似度 + 图位置 + 单元改动，文件与单元两层（仅报告）。代价 = 一个改动窗口加一次全量索引，需数分钟；进度走 stderr
join.days	改动窗口天数
structure	树尺度结构判决：熵、判轴与发现，经核 structure/1（仅报告）
structure.deep	同时按目录卷积克隆块与死单元并判 S6 冗余轴（会跑去冗普查与存活判决；缺省 = 该轴诚实未判）
structure.days	按此 git 窗口天数判 S5 文档陈旧轴（所引代码晚于文档最后编辑而变；缺省 = 该轴诚实未判）
structure.split_candidates	为每个越过冻结软线的判决文件计一次拆分价：给出最优缝与 ROI，或以数字说明文件为何内聚的豁免
trend	主线历史上的分数轨迹：逐提交绝对检查分，缓存于索引，可重建。每个未缓存提交都是临时工作树里的一次完整 check——冷跑请用 --batch 限量；进度走 stderr
trend.commits	主线窗口：最新 N 个第一父提交
trend.batch	每次运行至多测量这么多未缓存提交（缺省 = 全部；GUI 传小批量以显示进度）
erase	确定性两段式擦除：经核 erase/1 只计划可证安全消除的行；默认演练
erase.apply	真正擦除计划所列内容（要求 git 仓库、干净工作区、目标未变；默认为演练）
erase.check	门模式：计划含任何可擦行即退出 1（本仓库以此自净）
check	棘轮门：对 ce-baseline.json 判决仓库 — 棘轮或 --fail-under 地板，任一独立可判负
check.days	改动窗口天数（省略 = 改动表保持为空）
check.fail_under	分数落在此千分比地板之下即判负
check.roast	在控制台判决后附一行毒舌评语（彩蛋）
baseline	把核的 newBaseline 落盘为 ce-baseline.json（无 CE_ACCEPT_BASELINE=1 时违规集只准缩）
baseline.days	改动窗口天数（省略 = 改动表保持为空）
dedup	经 winnowing 指纹索引检测 T1/T2 克隆
dedup.path	要索引的目录（默认当前目录）
dedup.db	索引数据库路径（默认 <path>/.ce/index.db）
dedup.min_tokens	报告阈值（归一化 token 数；默认 winnowing 保证阈值 50）
dedup.min_distinct	多样性地板：唯一 token 数更少的块被抑制（默认 7，实测校准；0 关闭）
dedup.check	只缩棘轮：克隆块超过 ce.toml [dedup] 预算即退出 1（比较即核的判决）
dedup.core	ce-core 可执行文件路径（仅 --check 咨询；默认：CE_CORE_BIN、与本二进制同目录的 ce-core、再 PATH）
daemon	前台运行按项目守护进程；通常由 `ce ping` / 钩子探针惰启
daemon.root	要服务的项目根
ping	经项目守护进程往返一次 ping（会惰启它）
ping.root	项目根（默认当前目录）
probe	PreToolUse 廉价门：从 stdin 读钩子信封，探守护进程，按 ce.toml [guard] 发权限决定
probe.hook	钩子模式：从 stdin 读 JSON 信封（必带）
audit	Stop 审计 v1：净行数 + 触及改动文件的重复块（仅 deny 档拦停）
audit.hook	钩子模式：从 stdin 读 JSON 信封（必带）
health	SessionStart 健康行 + 守护进程预热
health.hook	钩子模式：从 stdin 读 JSON 信封（必带）
precommit	pre-commit 门：暂存净行数 + 触及的重复（deny 档触重即退出 1）
precommit.root	仓库根（默认当前目录）
mcp	stdio 上的 MCP 服务器：每个判决家族的只读报告面
mcp.root	工具作用的项目根（默认当前目录）
eject	卸载项目状态：.ce/、基线、钉扎件（默认试运行）
eject.root	要卸载的项目根（默认当前目录）
eject.yes	真正移除（默认试运行并点名每个目标）
";

fn zh_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| ZH_TSV.lines().filter_map(|l| l.split_once('\t')).collect())
}

/// One arg's zh help: the command's own key, else the shared
/// `judge.<id>` fallback. JudgeArgs is flattened into SEVEN commands
/// and the table carried a private copy of each of its three help
/// lines per command — twenty-one strings, and every copy of --core
/// had dropped the resolution order (CE_CORE_BIN, sibling, PATH) the
/// English derive states once. The `<cmd>.core` keys that remain are
/// independent clap declarations, not copies of this set, so they
/// keep their own row.
fn zh_help(m: &HashMap<&'static str, &'static str>, cmd: &str, id: &str) -> Option<&'static str> {
    m.get(format!("{cmd}.{id}").as_str())
        .or_else(|| m.get(format!("judge.{id}").as_str()))
        .copied()
}

/// Swap every about/help to zh off the lookup. Each subcommand is
/// touched exactly once (clap re-appends on mutation, so a second
/// pass would scramble the listing order); a lookup miss is a no-op
/// here — the completeness test below is the enforcement.
pub(crate) fn localize(mut cmd: clap::Command) -> clap::Command {
    let m = zh_map();
    cmd = cmd.about(m["ce"]).mut_arg("lang", |a| a.help(m["ce.lang"]));
    let subs: Vec<(String, Vec<String>)> = cmd
        .get_subcommands()
        .map(|s| {
            let args = s
                .get_arguments()
                .filter(|a| a.get_help().is_some())
                .map(|a| a.get_id().to_string())
                .collect();
            (s.get_name().to_string(), args)
        })
        .collect();
    for (name, args) in subs {
        cmd = cmd.mut_subcommand(&name, |mut sc| {
            if let Some(zh) = m.get(name.as_str()) {
                sc = sc.about(*zh);
            }
            for id in &args {
                if let Some(zh) = zh_help(m, &name, id) {
                    sc = sc.mut_arg(id.as_str(), |a| a.help(zh));
                }
            }
            sc
        });
    }
    cmd
}

#[cfg(test)]
#[path = "../tests/unit/main_lang.rs"]
mod tests;
