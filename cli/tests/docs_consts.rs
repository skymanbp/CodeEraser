mod docs_consts_stack;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Chip {
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
struct Def {
    file: PathBuf,
    name: String,
    value: String,
}

type Families = BTreeMap<String, Vec<Chip>>;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

fn page(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn between<'a>(text: &'a str, start: usize, end: usize, what: &str) -> &'a str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("consts parser: invalid {what} range"))
}

/// Find `pat` at or after `from`, or panic naming the surface — the
/// ONE owner of the find-or-refuse idiom both parsers lean on.
fn seek(text: &str, from: usize, pat: &str, what: &str) -> usize {
    text[from..]
        .find(pat)
        .map(|i| from + i)
        .unwrap_or_else(|| panic!("{what} has no {pat}"))
}

fn parse_chips(body: &str, label: &str, family: &str) -> Vec<Chip> {
    let mut chips = Vec::new();
    let mut item = 0;
    let ctx = format!("{label} family {family}: chip");
    while let Some(rel) = body[item..].find("<li><b>") {
        let name_start = item + rel + "<li><b>".len();
        let name_end = seek(body, name_start, "</b>", &ctx);
        let value_start = name_end + "</b>".len();
        let value_end = seek(body, value_start, "</li>", &ctx);
        chips.push(Chip {
            name: between(body, name_start, name_end, "chip name").to_string(),
            value: between(body, value_start, value_end, "chip value")
                .trim()
                .to_string(),
        });
        item = value_end + "</li>".len();
    }
    chips
}

fn parse_page(text: &str, label: &str) -> Families {
    let mut families = BTreeMap::new();
    let mut cursor = 0;
    while let Some(rel) = text[cursor..].find(r#"<ul class="consts">"#) {
        let ul = cursor + rel;
        let end = seek(text, ul, "</ul>", &format!("{label}: consts block"));
        let before = &text[..ul];
        let heading = before
            .rfind(r#"<h3><span class="n">"#)
            .unwrap_or_else(|| panic!("{label}: consts block with no preceding numbered heading"));
        let n_start = heading + r#"<h3><span class="n">"#.len();
        let n_end = seek(
            text,
            n_start,
            "</span>",
            &format!("{label}: numbered heading"),
        );
        let family = between(text, n_start, n_end, "family number").to_string();
        assert!(
            !family.is_empty() && family.bytes().all(|b| b.is_ascii_digit()),
            "{label}: consts block heading is not numbered"
        );
        let body = between(
            text,
            ul + r#"<ul class="consts">"#.len(),
            end,
            "consts body",
        );
        assert!(
            families
                .insert(family.clone(), parse_chips(body, label, &family))
                .is_none(),
            "{label}: duplicate family {family}"
        );
        cursor = end + "</ul>".len();
    }
    families
}
fn source_files(root: &Path, dir: &str, ext: &str) -> Vec<PathBuf> {
    fn visit(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read source dir {dir:?}: {e}")) {
            let path = entry.expect("source directory entry").path();
            if path.is_dir() {
                visit(&path, ext, out);
            } else if path.extension().is_some_and(|x| x == ext) {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    visit(&root.join(dir), ext, &mut out);
    out.sort();
    out
}

/// The ONE walk-and-parse shell both languages share; the per-line
/// grammar is the only thing that differs, so it is the parameter.
fn defs_in(root: &Path, dir: &str, ext: &str, parse: fn(&str) -> Option<(&str, &str)>) -> Vec<Def> {
    source_files(root, dir, ext)
        .into_iter()
        .flat_map(|file| {
            let text = fs::read_to_string(&file).expect("read source file");
            text.lines()
                .filter_map(parse)
                .map(|(name, value)| Def {
                    file: file.clone(),
                    name: name.to_string(),
                    value: value.to_string(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn rust_line(line: &str) -> Option<(&str, &str)> {
    let marker = line.find("const ")? + "const ".len();
    let rest = &line[marker..];
    let name_len = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    let value = rest.split_once('=')?.1.trim().trim_end_matches(';').trim();
    (!value.is_empty()).then_some((&rest[..name_len], value))
}

fn haskell_line(line: &str) -> Option<(&str, &str)> {
    let rest = line.trim_start();
    let name_len = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    let after = rest[name_len..].trim_start();
    let value = after.strip_prefix('=')?.trim();
    (name_len > 0 && !value.is_empty()).then_some((&rest[..name_len], value))
}

fn normalize(mut value: String) -> String {
    value = value.replace('_', "");
    while value.ends_with(".0") {
        value.truncate(value.len() - 2);
    }
    value
}

fn first_number(value: &str) -> Option<String> {
    let start = value
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i)?;
    let tail = &value[start..];
    let end = tail
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit() || *c == '_' || *c == '.')
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(1);
    Some(normalize(tail[..end].to_string()))
}

fn allowlist() -> BTreeSet<&'static str> {
    // "since proto": the erase family's wire BIRTH version — owned by
    // VERSIONING.md's 2.16.0 entry, not by any source binding
    "kgram\nwindow\nguarantee t\nnear-miss band\nschema\nscale\nsizeHard H\n[softMin, softMax]\nsizeCeil fallback\nlegs cross at\ntsconfig extends\nprecision gate\nrewriteNum/Den\ntolNum/tolDen\nseamSoft S\nseamHard H\nseamPMax\nknob codes\ndestFloor\nrow + knob cap\nfull-scale grid\ndeclineFloorMicro\nfile_lines_warn S\nfile_lines_fail H\nzone_tiers\nFPR gate\nfeed schema\nclasses\nreason codes\nwire\nminPoints\nsince proto"
        .lines()
        .collect()
}

/// Collision routing: (owning file, source binding) for the chip
/// names that resolve differently per family; None file = global.
fn collision<'a>(family: &str, name: &'a str) -> (Option<&'static str>, &'a str) {
    match (family, name) {
        ("04", "violCost") => (Some("core/app/CE/Structure/Cost.hs"), "structViolCost"),
        ("05", "violCost") => (Some("core/app/CE/Verdict/Cost.hs"), name),
        ("05" | "06", "sccFloor") => (Some("core/app/CE/Graph/Cost.hs"), name),
        ("06" | "07", "entryMask") => (Some("core/app/CE/Graph/Cost.hs"), name),
        _ => (None, name),
    }
}

fn defs_for<'a>(defs: &'a [Def], family: &str, name: &str) -> Vec<&'a Def> {
    let (mapped, wanted) = collision(family, name);
    defs.iter()
        .filter(|d| d.name == wanted)
        .filter(|d| mapped.is_none_or(|p| d.file.to_string_lossy().replace('\\', "/").ends_with(p)))
        .collect()
}

fn value_for(def: &Def, defs: &[Def], seen: &mut BTreeSet<String>) -> Option<String> {
    if let Some(value) = first_number(&def.value) {
        return Some(value);
    }
    let alias = def.value.rsplit("::").next()?.trim();
    if !seen.insert(alias.to_string()) {
        return None;
    }
    let next = defs.iter().find(|d| d.name == alias)?;
    value_for(next, defs, seen)
}

fn assert_sources(root: &Path, families: &Families) {
    let mut defs = defs_in(root, "cli/src", "rs", rust_line);
    defs.extend(defs_in(root, "core/app", "hs", haskell_line));
    let allow = allowlist();
    for (family, chips) in families {
        for chip in chips {
            if allow.contains(chip.name.as_str()) {
                continue;
            }
            let matches = defs_for(&defs, family, &chip.name);
            assert_eq!(
                matches.len(),
                1,
                "family {family} chip {}: expected exactly one source constant, found {}",
                chip.name,
                matches.len()
            );
            let mut seen = BTreeSet::new();
            let source = value_for(matches[0], &defs, &mut seen).unwrap_or_else(|| {
                panic!(
                    "family {family} chip {}: source has no numeric value ({})",
                    chip.name, matches[0].value
                )
            });
            let documented = first_number(&chip.value).unwrap_or_else(|| {
                panic!(
                    "family {family} chip {}: value has no leading number ({})",
                    chip.name, chip.value
                )
            });
            assert_eq!(
                normalize(source),
                normalize(documented),
                "family {family} chip {}: source {} != documented {}",
                chip.name,
                matches[0].value,
                chip.value
            );
        }
    }
}

#[test]
fn how_page_constant_chips_are_locked_and_resolvable() {
    let root = repo_root();
    let en = parse_page(&page(&root, "site/how/index.html"), "EN");
    let zh = parse_page(&page(&root, "site/zh/how/index.html"), "ZH");
    assert_eq!(
        en.keys().collect::<BTreeSet<_>>(),
        zh.keys().collect::<BTreeSet<_>>()
    );
    for family in en.keys() {
        assert_eq!(
            en[family].len(),
            zh[family].len(),
            "family {family}: chip count drift"
        );
        for (i, (a, b)) in en[family].iter().zip(&zh[family]).enumerate() {
            // the VALUE is the leading numeric token; trailing prose is translated
            assert!(
                first_number(&a.value) == first_number(&b.value),
                "family {family} chip {i}: value drift ({} vs {})",
                a.value,
                b.value
            );
        }
    }
    let total: usize = en.values().map(Vec::len).sum();
    assert!(total > 80, "only {total} chips harvested; parser is broken");
    assert_sources(&root, &en);
}
