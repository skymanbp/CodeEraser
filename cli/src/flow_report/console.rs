//! The console face of the flow document (design booklet §5.4): the
//! degraded reason first when the core did not judge, one line per
//! finding (`path:line  unit  kind  var`, an advisory one marked as
//! such), the units left unjudged with their reasons, and one counts
//! line. Every sentence has its Chinese twin (i18n::line); the kind
//! names, paths and unit names stay as written.

use super::report::Report;
use crate::i18n::{line, t};

pub fn console(r: &Report) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(why) = &r.degraded {
        out.push(line(
            "flow: degraded — {} (no judgment)",
            "函数内死代码：已降级——{}（未判决）",
            &[why],
        ));
    }
    for f in &r.findings {
        let mark = if f.judged {
            ""
        } else {
            t("  (advisory)", "  （顾问）")
        };
        let var = f.var.as_deref().unwrap_or("-");
        out.push(format!(
            "{}:{}  {}  {}  {var}{mark}",
            f.path, f.line, f.unit, f.kind
        ));
    }
    for x in &r.refused {
        out.push(line(
            "unjudged {} {}: {}",
            "未判 {} {}：{}",
            &[&x.path, &x.unit, &x.reason],
        ));
    }
    let n = |k: &str| r.counts.get(k).copied().unwrap_or(0);
    out.push(line(
        "{} unit(s): {} finding(s), {} judged, {} shown; {} dynamic unit(s) skipped, {} unjudged",
        "{} 个单元：{} 条发现，{} 条判决，显示 {} 条；{} 个动态单元整单元跳过，{} 个未判",
        &[
            &n("units"),
            &n("findings"),
            &n("judged"),
            &n("shown"),
            &n("dynamicUnits"),
            &n("refused"),
        ],
    ));
    out
}
