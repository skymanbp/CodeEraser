//! The ONE ce.toml glob dialect — gitignore/override syntax as the
//! `ignore` crate compiles it — for every reader: the walk's exclude
//! list, the rulepack's class globs, and `[graph] entry_globs`, which
//! kept a hand-rolled three-form matcher (exact path / `dir/` prefix /
//! `*.ext`) until plan v2.18 step #14 — `src/**/*.ts` matched nothing
//! there, silently. Three refusals and one rewrite live here and
//! nowhere else:
//! - a leading `!` — an exclude entry is an exclusion already and a
//!   class or entry pattern an inclusion already, so a user `!` would
//!   double-negate into a silent no-op;
//! - a leading `#` — the dialect reads it as a COMMENT (`#root.ts`
//!   selected nothing where the legacy matcher matched the literal
//!   file);
//! - a `\` anywhere — an ESCAPE in this syntax, never a separator:
//!   the former `\`→`/` rewrite turned `literal\*.rs` (an escaped
//!   star) into a directory pattern, and spelled a Windows path into
//!   one that matched nothing on this project's primary platform.
//!   There is one legal spelling; the message carries the fix;
//! - for INCLUSION patterns only, a trailing `/` becomes `/**`: an
//!   override set is asked about FILE paths (`matched(path, false)`),
//!   which a directory-only pattern never matches, so the legacy
//!   `dir/` entry form — and a class glob `src/` — would select
//!   nothing. Exclusions keep `dir/`: the walker prunes the directory.

use ignore::overrides::{Override, OverrideBuilder};
use std::path::Path;

/// A compiled inclusion set: a path is selected iff it whitelists.
pub(crate) type Inclusions = Override;

/// One user-written glob into an override set; `exclude` selects the
/// direction. `what` names the ce.toml key in every refusal.
pub(crate) fn add_user_glob(
    builder: &mut OverrideBuilder,
    glob: &str,
    exclude: bool,
    what: &str,
) -> Result<(), String> {
    if glob.starts_with('!') {
        let already = if exclude { "exclusions" } else { "inclusions" };
        return Err(format!(
            "ce.toml {what} {glob}: write the glob without '!' (entries are {already} already)"
        ));
    }
    if let Some(rest) = glob.strip_prefix('#') {
        return Err(format!(
            "ce.toml {what} {glob}: a leading '#' reads as a comment in this syntax — spell the character as a class (`[#]{rest}`)"
        ));
    }
    if glob.contains('\\') {
        return Err(format!(
            "ce.toml {what} {glob}: '\\' is an escape in this syntax, not a separator — spell paths with '/' (a literal metacharacter is written as a class, `[*]`)"
        ));
    }
    let g = if !exclude && glob.ends_with('/') {
        format!("{glob}**")
    } else {
        glob.to_string()
    };
    let g = if exclude { format!("!{g}") } else { g };
    builder
        .add(&g)
        .map(|_| ())
        .map_err(|e| format!("ce.toml {what} {glob}: {e}"))
}

/// One inclusion set from a declared glob list, compiled once per
/// run; a pattern the dialect cannot read is a named error, never a
/// silent no-match.
pub(crate) fn compile_inclusions(
    root: &Path,
    globs: &[String],
    what: &str,
) -> Result<Inclusions, String> {
    let mut b = OverrideBuilder::new(root);
    for glob in globs {
        add_user_glob(&mut b, glob, false, what)?;
    }
    b.build().map_err(|e| format!("ce.toml {what}: {e}"))
}

/// Whether `rel` — the '/'-spelled repo-relative path every report
/// keys on — is selected by the set.
pub(crate) fn selected(set: &Inclusions, rel: &str) -> bool {
    matches!(
        set.matched(Path::new(rel), false),
        ignore::Match::Whitelist(_)
    )
}

#[cfg(test)]
#[path = "../../tests/unit/scan/globs.rs"]
mod tests;
