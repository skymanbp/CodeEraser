//! Lock the stack page's authored numeric facts across EN/ZH and
//! back to their owning source. Evaluation values are deliberately
//! absent: bench_render_dashboard owns those generated blocks.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has a parent")
        .to_path_buf()
}

fn constants(path: &Path) -> BTreeMap<String, String> {
    let page = std::fs::read_to_string(path).expect("read stack page");
    let mut found = BTreeMap::new();
    let (marker, end) = (r#"<span data-const=""#, "</span>");
    let mut rest = page.as_str();
    while let Some(start) = rest.find(marker) {
        let named = &rest[start + marker.len()..];
        let quote = named.find('"').expect("data-const closing quote");
        let name = &named[..quote];
        let body = &named[quote + 1..];
        let open = body.find('>').expect("data-const tag close") + 1;
        let close = body[open..].find(end).expect("data-const closing tag") + open;
        assert!(
            found
                .insert(name.to_string(), body[open..close].to_string())
                .is_none(),
            "duplicate stack constant {name}"
        );
        rest = &body[close + end.len()..];
    }
    found
}

fn source_contains(root: &Path, rel: &str, needle: &str) {
    let source = std::fs::read_to_string(root.join(rel)).expect("read constant source");
    assert!(source.contains(needle), "{rel} does not contain {needle:?}");
}

#[test]
fn stack_page_constants_are_locked_and_resolvable() {
    let root = root();
    let en = constants(&root.join("site/stack/index.html"));
    let zh = constants(&root.join("site/zh/stack/index.html"));
    assert_eq!(en, zh, "stack EN/ZH numeric facts drifted");
    assert_eq!(
        en,
        BTreeMap::from([
            ("ci-floor".into(), "950".into()),
            ("ghc".into(), "9.14.1".into()),
            ("proto".into(), "3.2.0".into()),
        ])
    );
    source_contains(&root, "cli/src/corelink.rs", r#"PROTO: &str = "3.2.0""#);
    source_contains(
        &root,
        ".github/workflows/ci.yml",
        r#"ghc-version: "9.14.1""#,
    );
    source_contains(&root, ".github/workflows/ci.yml", "--fail-under 950");
    let site_svg = std::fs::read(root.join("site/assets/stack.svg")).expect("site stack svg");
    let docs_svg = std::fs::read(root.join("docs/assets/stack.svg")).expect("docs stack svg");
    assert_eq!(site_svg, docs_svg, "site/docs stack diagrams differ");
    let svg = String::from_utf8(site_svg).expect("stack svg utf8");
    for value in en.values() {
        assert!(svg.contains(value), "stack diagram omits constant {value}");
    }
}
