//! A path a user typed, spelled the way the index spells one (release
//! 1.10.0). Every face argument that names an indexed file —
//! `ce similar --at`, `ce arch --impact`, their MCP arguments and GUI
//! inputs — and the declared directories of `[graph] search_roots` read
//! through this one function, so `./a.rs`, `.//a.rs`, `a\b.rs` and the
//! absolute path under the root name one file on every face.

use std::path::{Component, Path, PathBuf};

/// `path` as `walk::rel_str` spells an indexed path: `\` read as `/`, an
/// absolute path under `root` made relative to it, every `.` component
/// and every repeated or trailing `/` dropped; `.` alone is the root,
/// spelled empty. A `..` stays (it names nothing inside the tree), and so
/// does an absolute path outside the root, slashed: neither matches an
/// indexed file, so the face's own "names no file" answer stands.
pub fn typed(root: &Path, path: &str) -> String {
    let slashed = path.replace('\\', "/");
    let given = Path::new(&slashed);
    let rel = match (given.is_absolute(), std::path::absolute(root)) {
        (false, _) => plain(given),
        (true, Ok(base)) => match plain(given).strip_prefix(plain(&base)) {
            Ok(inside) => inside.to_path_buf(),
            Err(_) => return slashed,
        },
        (true, Err(_)) => return slashed,
    };
    rel.components()
        .filter_map(|c| match c {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// The path without its `.` components (what `components` would keep of
/// `a/./b` and `./a`, so two spellings of one place strip as prefixes).
fn plain(path: &Path) -> PathBuf {
    path.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/scan/walk/typed.rs"]
mod tests;
