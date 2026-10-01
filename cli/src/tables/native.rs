//! Today's Rust definition tables in the `tables/1` package's shape
//! (VERSIONING 7.7.0). Step 2 deletes this file: it exists only so the
//! equivalence gate (cli/tests/it/tables_equivalence.rs) can prove the
//! core's transcription equals the Rust text it was taken from. Every
//! value here is read off the table its consumers read today — never
//! restated — and a text table is cut the way its own reader cuts it.

use crate::scan::lang::{LANGS, Lang};
use serde_json::{Value, json};

/// The package: one key per table family (the response's content keys).
pub fn native_pack() -> Value {
    use crate::docdup::spec as d;
    use crate::graph::{compdb_flags as c, deadcode::flags as f, keys as k};
    use crate::scan::walk as w;
    let mut pack = json!({
        "languages": languages(),
        "scan": by_lang(|l| json!(crate::scan::spec::spec(l))),
        "flow": by_lang(|l| json!(crate::flow::spec::spec(l))),
        "slot": by_lang(|l| json!(crate::merge::slot::slot_spec(l))),
        "sites": by_lang(|l| json!(crate::graph::spec::sites(l))),
        "calls": calls(),
        "fourclass": fourclass(),
        "ladder": ladder(),
        "walk": {"secret_globs": w::SECRET_GLOBS, "builtin_excludes": w::BUILTIN_EXCLUDES},
        "outputs": rows(crate::scan::outputs::OUTPUTS),
        "docdup": docdup(),
        "keys": {"config_names": k::CONFIG_NAMES, "twin_exts": k::TWIN_EXTS},
        "flags": {"entry_names": ws(f::ENTRY_NAMES), "entry_dirs": rows(f::ENTRY_DIRS)},
        "tombstone": tombstone(),
        "compdb": {"gnu": ws(c::GNU), "skip": ws(c::SKIP)},
        "protocol": protocol(),
    });
    // the two per-language entries of a common family, and docdup's
    // numbers — grafted, as the core grafts them from CE.Docdup.Cost
    pack["fourclass"]["extra"] = by_lang(|l| json!(crate::fourclass::kinds::extra(l)));
    pack["docdup"]["doc_spec"] = by_lang(|l| json!(d::doc_spec(l)));
    for (key, n) in [
        ("min_doc_tokens", json!(d::MIN_DOC_TOKENS)),
        ("license_head_lines", json!(d::LICENSE_HEAD_LINES)),
        ("verbatim_floor", json!(d::VERBATIM_FLOOR)),
        ("doc_shingle", json!(d::DOC_SHINGLE)),
        ("doc_line_cap", json!(d::DOC_LINE_CAP)),
    ] {
        pack["docdup"][key] = n;
    }
    pack
}

/// The judged languages, the ones a per-language table answers for.
fn judged() -> impl Iterator<Item = Lang> {
    LANGS
        .iter()
        .map(|&(l, ..)| l)
        .filter(|&l| (Lang::judged_mask() >> (l as i64)) & 1 == 1)
}

/// One value per judged language, keyed by its report name.
fn by_lang(f: impl Fn(Lang) -> Value) -> Value {
    Value::Object(judged().map(|l| (l.name().to_string(), f(l))).collect())
}

/// A whitespace-separated name table, as its readers cut it.
fn ws(table: &str) -> Vec<&str> {
    table.split_whitespace().collect()
}

/// A row table: one row per line, its first word the head, the rest
/// its words — the shape `outputs::manifests` and its kin read.
fn rows(table: &str) -> Vec<(&str, Vec<&str>)> {
    table
        .lines()
        .filter_map(|row| {
            let mut words = row.split_whitespace();
            Some((words.next()?, words.collect()))
        })
        .collect()
}

fn languages() -> Value {
    let rows: Vec<Value> = LANGS
        .iter()
        .map(|&(l, exts, name, scan_only)| {
            json!({
                "code": l as i64,
                "name": name,
                "exts": exts,
                "scan_only": scan_only,
                "prose_only": l.prose_only(),
                "judged": (Lang::judged_mask() >> (l as i64)) & 1 == 1,
                "document": matches!(l, Lang::Markdown | Lang::Html),
            })
        })
        .collect();
    json!({
        "rows": rows,
        "machine_txt": Lang::MACHINE_TXT,
        "mention_whole_run_exts": crate::scan::lang::MENTION_WHOLE_RUN_EXTS,
    })
}

fn calls() -> Value {
    use crate::graph::spec::{R_FORMALS, calls, protected};
    let formals: Vec<(&str, Vec<&str>)> = R_FORMALS
        .lines()
        .filter_map(|row| row.split_once(": "))
        .map(|(callee, list)| (callee, ws(list)))
        .collect();
    json!({
        "calls": by_lang(|l| json!(calls(l))),
        "protected": by_lang(|l| json!(protected(l))),
        "r_formals": formals,
    })
}

fn fourclass() -> Value {
    use crate::fourclass::kinds as k;
    json!({
        "redeclaring": k::REDECLARING,
        "package_level": k::PACKAGE_LEVEL,
        "java_fields": k::JAVA_FIELDS,
        "c_variable": k::C_VARIABLE,
        "ts_lexical": k::TS_LEXICAL,
        "bodied": k::BODIED,
    })
}

fn ladder() -> Value {
    use crate::graph::ladder as g;
    json!({
        "hs": {"boot": g::hs_boot::BOOT.iter().map(|&(p, m)| (p, ws(m))).collect::<Vec<_>>()},
        "java": {"packages": ws(g::java_jdk::PACKAGES), "lang": ws(g::java_jdk::LANG)},
        "ts": {
            "builtins": ws(g::ts::node::BUILTINS),
            "prefix_only": ws(g::ts::node::PREFIX_ONLY),
        },
        "go": {"std": ws(g::go::STD)},
        "py": {"stdlib": ws(g::py::STDLIB)},
        "lua": {"stdlib": ws(g::lua::STDLIB)},
        "rs": {"builtin": g::rs::rs_use::BUILTIN},
    })
}

fn docdup() -> Value {
    use crate::docdup::spec as d;
    json!({
        "license_markers": d::LICENSE_MARKERS.split('|').collect::<Vec<_>>(),
        "skeleton_prefixes": d::SKELETON_PREFIXES.split('|').collect::<Vec<_>>(),
        "allow_marker": d::ALLOW_MARKER,
        "kind_names": d::KIND_NAMES,
    })
}

fn tombstone() -> Value {
    use crate::tombstone::vocab as v;
    let bars = |t: &'static str| v::entries(t).collect::<Vec<_>>();
    let chars = |cs: &[char]| cs.iter().map(char::to_string).collect::<Vec<_>>();
    json!({
        "negations": bars(v::NEGATIONS),
        "keywords": bars(v::KEYWORDS),
        "en_prefix": bars(v::EN_PREFIX),
        "en_suffix": bars(v::EN_SUFFIX),
        "zh_prefix": bars(v::ZH_PREFIX),
        "zh_suffix": bars(v::ZH_SUFFIX),
        "marks_en": bars(v::MARKS_EN),
        "marks_zh": bars(v::MARKS_ZH),
        "stop_en": bars(v::STOP_EN),
        "min_ascii_name": v::MIN_ASCII_NAME,
        "min_wide_name": v::MIN_WIDE_NAME,
        "join_max": v::JOIN_MAX,
        "open": chars(v::OPEN),
        "close": chars(v::CLOSE),
    })
}

fn protocol() -> Value {
    use crate::mention::conv::protocol as p;
    let by_stem: Vec<(Vec<&str>, Vec<&str>)> = p::TS_BY_STEM
        .lines()
        .filter_map(|row| row.split_once(" : "))
        .map(|(stems, names)| (ws(stems), ws(names)))
        .collect();
    json!({
        "py_names": ws(p::PY_NAMES),
        "py_prefixes": p::PY_PREFIXES,
        "ts_by_stem": by_stem,
        "ts_pages": ws(p::TS_PAGES),
        "ts_routes": ws(p::TS_ROUTES),
        "java_names": ws(p::JAVA_NAMES),
        "c_names": ws(p::C_NAMES),
        "c_prefixes": p::C_PREFIXES,
        "lua_names": ws(p::LUA_NAMES),
        "love_names": ws(p::LOVE_NAMES),
        "r_names": ws(p::R_NAMES),
    })
}
