//! The flow class's leg of the Stop audit and the git-hook faces (plan
//! v2.31 step 5; design booklet §5.4): every changed file's after side
//! the tombstone leg already read (one git batch for every text leg),
//! lowered and judged once over the audit's core link (flow/1), the
//! feed's `flow` object counting what the core found — files, units,
//! findings, the findings by kind, the judged ones. Counts only: the
//! sites are `ce flow`'s. Never a block at any tier — the Stop is not
//! this class's decision point, and §4.2 promotes nothing without a
//! record. No changed unit = no key; no core = the object names why.

use crate::corelink::Link;
use crate::flow::lower::{Lowered, lower_file};
use crate::flow_report::{judged, kinds_json};
use crate::tombstone::texts::Loaded;
use serde_json::{Value, json};

pub(super) fn leg(loaded: &[Loaded], link: Option<&mut Link>) -> Option<Value> {
    let files: Vec<Lowered> = loaded
        .iter()
        .filter_map(|l| lower_file(&l.after, l.lang))
        .filter(|f| !f.units.is_empty())
        .collect();
    if files.is_empty() {
        return None;
    }
    let units: usize = files.iter().map(|f| f.units.len()).sum();
    let mut v = json!({ "files": files.len(), "units": units });
    let verdict = match link {
        Some(l) => crate::flow::wire::judge(l, &files),
        None => Err("core unavailable".into()),
    };
    match verdict {
        Ok(verdict) => {
            let found = || {
                verdict
                    .findings
                    .iter()
                    .map(|(f, _, x)| (files[*f].lang, x.kind))
            };
            v["findings"] = json!(verdict.findings.len());
            v["kinds"] = kinds_json(found().map(|(_, k)| k));
            v["judged"] = json!(found().filter(|(lang, k)| judged(*lang, *k)).count());
        }
        Err(why) => v["degraded"] = json!(why),
    }
    Some(v)
}
