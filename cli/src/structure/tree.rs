//! Tree aggregation from walked relative paths: dense directory
//! nodes with parent links, per-directory fanout and depth, the
//! sibling-set NAME-SHAPE distributions (S1 axis input — since 7.2.0
//! the seven character-class facts of each stem cross the wire as
//! shape bits and the style decision is the core's,
//! CE.Structure.Shape; names never cross), and the convention bits
//! (S4: README / config presence). Deterministic by construction:
//! directories are discovered in sorted path order and numbered
//! densely, so the same walk always yields the same shape.

use std::collections::BTreeMap;

/// One directory node. `parent` = self for the root (dense id 0).
pub struct Dir {
    pub parent: usize,
    pub depth: u32,
    /// Immediate child directories.
    pub subdirs: u32,
    /// Immediate files.
    pub files: u32,
    /// S1 input: file count per stem shape (the `shape_bits` key),
    /// ascending by construction — the `patternShapes` rows.
    pub shapes: BTreeMap<u8, u32>,
    /// S4 input: convention bits (see CONVENTIONS).
    pub conventions: u8,
}

/// The aggregated tree: dense ids, root = 0 (the walk root). `ids`
/// maps directory paths to their dense id — the join key every
/// downstream aggregation (dir edges, rollups) resolves through, so
/// the numbering can never fork.
pub struct Tree {
    pub dirs: Vec<Dir>,
    pub ids: BTreeMap<String, usize>,
}

/// Convention bits (S4): bit 0 = a README.* lives here, bit 1 = a
/// recognized config basename lives here.
const CONV_README: u8 = 1;
const CONV_CONFIG: u8 = 2;

/// Recognized config basenames — the presence FACT for the S4 axis
/// (which directories carry their own configuration), not a policy.
const CONFIG_NAMES: [&str; 8] = [
    "ce.toml",
    "Cargo.toml",
    "pyproject.toml",
    "package.json",
    "go.mod",
    "Makefile",
    "CMakeLists.txt",
    "flake.nix",
];

/// Build the aggregate tree from walk-relative file paths (forward
/// slashes, the walk::rel_str spelling). Every ancestor directory
/// of every file becomes a node; the root is the empty prefix.
pub fn build(paths: &[String]) -> Tree {
    let mut ids: BTreeMap<String, usize> = BTreeMap::new();
    let mut dirs: Vec<Dir> = vec![root()];
    ids.insert(String::new(), 0);
    let mut sorted: Vec<&String> = paths.iter().collect();
    sorted.sort();
    for path in sorted {
        let (dir_path, name) = split_dir(path);
        let dir = ensure_dir(&mut ids, &mut dirs, dir_path);
        dirs[dir].files += 1;
        *dirs[dir].shapes.entry(shape_bits(stem(name))).or_insert(0) += 1;
        let upper = name.to_ascii_uppercase();
        if upper == "README" || upper.starts_with("README.") {
            dirs[dir].conventions |= CONV_README;
        }
        if CONFIG_NAMES.contains(&name) {
            dirs[dir].conventions |= CONV_CONFIG;
        }
    }
    Tree { dirs, ids }
}

/// The owning directory id of a walk-relative FILE path (the same
/// split every aggregation uses); None = the file's directory never
/// entered this tree — a caller-side universe mismatch, never a
/// guess.
pub fn dir_of(tree: &Tree, path: &str) -> Option<usize> {
    let (dir_path, _) = split_dir(path);
    tree.ids.get(dir_path).copied()
}

fn root() -> Dir {
    Dir {
        parent: 0,
        depth: 0,
        subdirs: 0,
        files: 0,
        shapes: BTreeMap::new(),
        conventions: 0,
    }
}

fn split_dir(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(i) => (&path[..i], &path[i + 1..]),
        None => ("", path),
    }
}

fn stem(name: &str) -> &str {
    match name.find('.') {
        Some(0) => &name[1..], // dotfiles classify by their tail
        Some(i) => &name[..i],
        None => name,
    }
}

/// Dense id for `dir_path`, creating the chain of ancestors on
/// first sight (each creation increments the parent's subdir count
/// exactly once — the id map is the visited set).
fn ensure_dir(ids: &mut BTreeMap<String, usize>, dirs: &mut Vec<Dir>, dir_path: &str) -> usize {
    if let Some(&id) = ids.get(dir_path) {
        return id;
    }
    let (parent_path, _) = split_dir(dir_path);
    let parent = ensure_dir(ids, dirs, parent_path);
    let id = dirs.len();
    dirs.push(Dir {
        parent,
        depth: dirs[parent].depth + 1,
        subdirs: 0,
        files: 0,
        shapes: BTreeMap::new(),
        conventions: 0,
    });
    dirs[parent].subdirs += 1;
    ids.insert(dir_path.to_string(), id);
    id
}

/// The seven stem facts as shape bits (7.2.0, plan v2.30 step 7b —
/// frozen positions, the same seven CE.Structure.Shape reads): 0 an
/// underscore / 1 a dash / 2 a lowercase letter / 3 an uppercase
/// letter / 4 digit-led / 5 first char uppercase / 6 unclassifiable
/// (an empty stem, or a char outside letters, digits, dash and
/// underscore). Collection only: the style decision these used to
/// feed here — a STYLE table over the low four bits, camel split from
/// pascal by the first char, digit-led and unclassifiable first — is
/// judgment and moved to the core with its truth table.
pub fn shape_bits(s: &str) -> u8 {
    let mut bits = 0u8;
    for c in s.chars() {
        bits |= match c {
            '_' => 1,
            '-' => 2,
            'a'..='z' => 4,
            'A'..='Z' => 8,
            '0'..='9' => 0,
            _ => 64,
        };
    }
    match s.chars().next() {
        None => bits | 64,
        Some(c) if c.is_ascii_digit() => bits | 16,
        Some(c) if c.is_ascii_uppercase() => bits | 32,
        Some(_) => bits,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/structure/tree.rs"]
mod tests;
