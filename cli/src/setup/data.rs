//! This plugin's data folder, when the environment names one — the
//! one reader of `CLAUDE_PLUGIN_DATA` on the measuring side. Claude
//! Code names the folder `plugins/data/<id>` and exports it to hook
//! commands; a Bash-tool shell has been observed holding ANOTHER
//! plugin's folder in the same variable. So the folder counts only
//! when its last path segment is the plugin's name or that name and
//! `-` — the rule the starter (`plugin/bin/ce.sh`) applies before it
//! execs `ce`. The search beside a foreign folder is the starter's
//! alone: here a foreign folder reads as no folder at all.

use std::ffi::OsStr;
use std::path::PathBuf;

/// The plugin's name: the part of `<plugin>@<marketplace>` before `@`.
fn plugin_name() -> &'static str {
    let id = super::NAMES.plugin;
    id.split_once('@').map_or(id, |(name, _)| name)
}

/// The folder the environment names, when it is this plugin's own.
pub fn plugin_data() -> Option<PathBuf> {
    own(std::env::var_os("CLAUDE_PLUGIN_DATA").as_deref())
}

/// The rule over one value, pure (the unit legs drive it): one
/// trailing separator dropped, either separator splits (the value may
/// be a native Windows path), an unset or empty value names nothing.
pub fn own(value: Option<&OsStr>) -> Option<PathBuf> {
    let value = value?;
    let text = value.to_string_lossy();
    let trimmed = text.strip_suffix('/').unwrap_or(&text);
    let trimmed = trimmed.strip_suffix('\\').unwrap_or(trimmed);
    let leaf = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    let name = plugin_name();
    let ours = leaf == name
        || leaf
            .strip_prefix(name)
            .is_some_and(|rest| rest.starts_with('-'));
    ours.then(|| PathBuf::from(value))
}

#[cfg(test)]
#[path = "../../tests/unit/setup/data.rs"]
mod tests;
