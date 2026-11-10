//! The reply's per-site extras of resolve/1 (proto 9.1.0, lane
//! v235_graphtrust): the `also` rows a Python site draws beside its
//! answer and the `opaque` star-import sites, each read strictly and
//! refused by name when absent or out of shape (moved out of mod.rs at
//! the E01 300-line line).

use crate::corelink::judged;
use crate::graph::ladder::{Also, Outcome, Rung};
use serde_json::Value;

/// The reply's `also` table (proto 9.1.0) by site: `[site, target,
/// rung]` strictly ascending by (site, target), never the site's own
/// answer. An absent table is a core that predates it — refused by name,
/// since reading it as empty would quietly drop the edges it carries.
pub(super) fn also_rows(reply: &Value, out: &[Outcome]) -> Result<Vec<Also>, String> {
    if reply.get("also").is_none() {
        return Err("resolve/1: no `also` table: a core before proto 9.1.0".into());
    }
    let rows: Vec<(usize, String, Rung)> = judged::table(reply, "also")?;
    let mut by_site: Vec<Also> = vec![Also::default(); out.len()];
    for (i, w) in rows.windows(2).enumerate() {
        if (w[0].0, &w[0].1) >= (w[1].0, &w[1].1) {
            return Err(format!(
                "resolve/1: wire skew: also {}: not after the row before it",
                i + 1
            ));
        }
    }
    for (site, target, rung) in rows {
        let primary = match out.get(site) {
            None => return Err(format!("resolve/1: wire skew: also names site {site}")),
            Some(Outcome::Resolved { path, .. }) => Some(path),
            Some(_) => None,
        };
        if primary == Some(&target) {
            return Err(format!(
                "resolve/1: wire skew: also repeats site {site}'s answer"
            ));
        }
        by_site[site].rows.push((target, rung));
    }
    opaque_rows(reply, &mut by_site)?;
    Ok(by_site)
}

/// The reply's `opaque` table (lane v235_graphtrust): the star-import
/// sites, strictly ascending, whose package's `__all__` no literal
/// reads — each carries one more unresolved ledger row. An absent table
/// is a core that predates it, refused by name: reading it as empty
/// would let such a site vouch for its language.
fn opaque_rows(reply: &Value, by_site: &mut [Also]) -> Result<(), String> {
    if reply.get("opaque").is_none() {
        return Err("resolve/1: no `opaque` table: a core before lane v235_graphtrust".into());
    }
    let rows: Vec<usize> = judged::table(reply, "opaque")?;
    if rows.windows(2).any(|w| w[0] >= w[1]) {
        return Err("resolve/1: wire skew: opaque rows not ascending".into());
    }
    for site in rows {
        let Some(also) = by_site.get_mut(site) else {
            return Err(format!("resolve/1: wire skew: opaque names site {site}"));
        };
        also.opaque = true;
    }
    Ok(())
}
