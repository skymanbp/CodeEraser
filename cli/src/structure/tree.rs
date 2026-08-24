//! Tree aggregation from walked relative paths: dense directory
//! nodes with parent links, per-directory fanout and depth, the
//! sibling-set NAME-PATTERN distributions (S1 axis input — pattern
//! CODES cross the wire, names never do), and the convention bits
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
    /// S1 input: file-name pattern counts, indexed by PATTERNS code.
    pub patterns: [u32; PATTERN_COUNT],
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

/// Name-pattern codes, index = wire code (frozen positions, the
/// wire.rs edge-code discipline). Classification looks at the file
/// STEM (up to the first dot); the style vocabulary covers the
/// conventions the scanned ecosystems actually use. Codes 0..6 =
/// lower_snake / lower_kebab / camel / pascal / upper_snake /
/// digit_led / other — dense positions on the wire; the label
/// STRINGS never travel and had no reader anywhere (the sweep's one
/// clean kill), so the vocabulary lives here as prose.
pub const PATTERN_COUNT: usize = 7;

/// Convention bits (S4): bit 0 = a README.* lives here, bit 1 = a
/// recognized config basename lives here.
pub const CONV_README: u8 = 1;
pub const CONV_CONFIG: u8 = 2;

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
        dirs[dir].patterns[pattern_code(stem(name)) as usize] += 1;
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
        patterns: [0; PATTERN_COUNT],
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
        patterns: [0; PATTERN_COUNT],
        conventions: 0,
    });
    dirs[parent].subdirs += 1;
    ids.insert(dir_path.to_string(), id);
    id
}

/// The stem's character-class flags — COLLECTION only; the decision
/// lives in pattern_code (split keeps both halves under the repo's
/// own cyclomatic gate, which caught the fused form at CC 25).
struct StemFlags {
    upper: bool,
    lower: bool,
    digit: bool,
    dash: bool,
    under: bool,
    ascii: bool,
}

fn flags_of(s: &str) -> StemFlags {
    let mut f = StemFlags {
        upper: false,
        lower: false,
        digit: false,
        dash: false,
        under: false,
        ascii: true,
    };
    for c in s.chars() {
        match c {
            'A'..='Z' => f.upper = true,
            'a'..='z' => f.lower = true,
            '0'..='9' => f.digit = true,
            '-' => f.dash = true,
            '_' => f.under = true,
            _ => f.ascii = false,
        }
    }
    f
}

/// The style decision as a TABLE (the DSL stance at home): index =
/// upper<<3 | lower<<2 | dash<<1 | under, value = pattern code.
/// Key 0b1100 encodes 3 (pascal) and is split to camel by first
/// char in pattern_code; every mixed-signal key answers 6 — the
/// repo's own scan gate caught the fused (CC 25), nested (CoC 16)
/// and flat-guard (CC 18) forms of this decision before it became
/// data.
const STYLE: [u8; 16] = [6, 0, 1, 6, 0, 0, 1, 6, 4, 4, 6, 6, 3, 6, 6, 6];

/// Pattern code of one stem (index into PATTERNS). Empty and
/// unclassifiable stems land on "other" — never a guess.
pub fn pattern_code(s: &str) -> u8 {
    let f = flags_of(s);
    if s.is_empty() || !f.ascii {
        return 6;
    }
    if f.digit && s.starts_with(|c: char| c.is_ascii_digit()) {
        return 5;
    }
    let key = (usize::from(f.upper) << 3)
        | (usize::from(f.lower) << 2)
        | (usize::from(f.dash) << 1)
        | usize::from(f.under);
    let code = STYLE[key];
    if code == 3 && !s.starts_with(|c: char| c.is_ascii_uppercase()) {
        return 2; // same flag key as pascal; camel splits by first char
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The classifier's vocabulary, one probe per PATTERNS row plus
    /// the refusal cases — codes are frozen wire positions.
    #[test]
    fn pattern_codes_cover_the_vocabulary() {
        let cases: [(&str, u8); 10] = [
            ("parse_result", 0),
            ("mod", 0),
            ("my-file", 1),
            ("parseResult", 2),
            ("ParseResult", 3),
            ("README", 4),
            ("MAX_LIMIT", 4),
            ("2026-08-17-notes", 5),
            ("mixed-and_under", 6),
            ("名字", 6),
        ];
        for (s, want) in cases {
            assert_eq!(pattern_code(s), want, "{s}");
        }
    }

    /// One small tree, every aggregate hand-checked: dense ids in
    /// sorted discovery order, parent/depth chains, fanouts, the
    /// pattern distribution and both convention bits.
    #[test]
    fn build_aggregates_a_small_tree_by_hand() {
        let paths: Vec<String> = [
            "README.md",
            "Cargo.toml",
            "src/lib.rs",
            "src/deep/one.rs",
            "src/deep/two-b.rs",
            "docs/Guide.md",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let t = build(&paths);
        assert_eq!(t.dirs.len(), 4, "root + docs + src + src/deep");
        let root = &t.dirs[0];
        assert_eq!((root.depth, root.subdirs, root.files), (0, 2, 2));
        assert_eq!(root.conventions, CONV_README | CONV_CONFIG);
        // sorted discovery: docs(1) < src(2) < src/deep(3)
        let (docs, src, deep) = (&t.dirs[1], &t.dirs[2], &t.dirs[3]);
        assert_eq!((docs.parent, docs.depth, docs.files), (0, 1, 1));
        assert_eq!((src.parent, src.subdirs, src.files), (0, 1, 1));
        assert_eq!((deep.parent, deep.depth, deep.files), (2, 2, 2));
        assert_eq!(deep.patterns[0], 1, "one.rs is lower_snake");
        assert_eq!(deep.patterns[1], 1, "two-b.rs is lower_kebab");
        assert_eq!(docs.patterns[3], 1, "Guide.md is pascal");
        assert_eq!(docs.conventions, 0, "no README, no config in docs/");
    }
}
