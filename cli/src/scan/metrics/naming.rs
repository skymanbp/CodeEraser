//! Readability naming FACTS (plan §4.1) plus the pinned conforms
//! mirror. Since 2.30.0 (ADR-008 batch-7 slice 14) the judgment —
//! which case family a language enforces, and who gets the godoc
//! underscore exemption — is the core's (CE.Scan.Cost.conforms);
//! this side extracts name-SHAPE facts for the wire and keeps
//! conforms only as the mirror the whole-report ensure proves equal
//! per run. Provenance: PEP 8 (Python), RFC 430 (Rust), Effective
//! Go + go vet's test-name rule, common TS practice.

use crate::scan::lang::Lang;
use crate::scan::spec::NameStyle;

/// Go test-function families where the toolchain itself accepts or
/// requires underscores (go vet "tests": TestType_Method subtables,
/// Example_suffix package examples).
const GO_TEST_PREFIXES: &[&str] = &["Test", "Benchmark", "Example", "Fuzz"];

/// The five naming facts as one scan.request `naming` row:
/// [lang, style, upper, under, test]. Sentinel names like
/// "(anonymous)" and non-identifier subjects (quoted / computed TS
/// object keys) carry style 0 — "no convention applies", said as a
/// fact. Leading underscores (unused/private markers) are tolerated
/// everywhere and trimmed before the shape reads.
pub fn facts(lang: Lang, style: NameStyle, name: &str) -> [i64; 5] {
    if name.starts_with(['(', '"', '\'', '[']) {
        return [lang as i64, 0, 0, 0, 0];
    }
    let core = name.trim_start_matches('_');
    [
        lang as i64,
        style_code(style),
        i64::from(core.contains(char::is_uppercase)),
        i64::from(core.contains('_')),
        i64::from(test_shape(core)),
    ]
}

/// Wire codes for the style column — frozen positions (0 = no
/// convention; the sentinel road shares it).
fn style_code(style: NameStyle) -> i64 {
    match style {
        NameStyle::Any => 0,
        NameStyle::Snake => 1,
        NameStyle::MixedCaps => 2,
    }
}

/// go vet's test-name boundary, stated as a shape fact: a family
/// prefix whose remainder does not START lowercase (empty,
/// uppercase, digit and `_` all qualify — Example_suffix is
/// godoc-legal). `Testing_helper` fails it: the old
/// starts_with("Test") exemption was wrong twice over — it exempted
/// this, and it exempted every mixedCaps language, not just Go.
/// Both defects die with the judgment's move to the core.
fn test_shape(core: &str) -> bool {
    GO_TEST_PREFIXES.iter().any(|p| {
        core.strip_prefix(p)
            .is_some_and(|rest| !rest.starts_with(char::is_lowercase))
    })
}

/// Pinned MIRROR of CE.Scan.Cost.conforms — the report face reads
/// it, and scan::analyze_judged's whole-report ensure proves it
/// equal to the core's verdict on every judged run. The godoc
/// exemption is gated on Go's OWN lang code: the same facts under
/// TypeScript or Haskell stay a violation.
pub fn conforms(row: [i64; 5]) -> bool {
    let [lang, style, upper, under, test] = row;
    match style {
        1 => upper == 0,
        2 => under == 0 || (lang == Lang::Go as i64 && test == 1),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn go(name: &str) -> [i64; 5] {
        facts(Lang::Go, NameStyle::MixedCaps, name)
    }

    #[test]
    fn snake_rules() {
        assert!(conforms(facts(
            Lang::Python,
            NameStyle::Snake,
            "load_config"
        )));
        assert!(conforms(facts(Lang::Python, NameStyle::Snake, "__init__")));
        assert!(!conforms(facts(
            Lang::Python,
            NameStyle::Snake,
            "loadConfig"
        )));
    }

    #[test]
    fn mixed_caps_rules() {
        assert!(conforms(go("loadConfig")));
        assert!(conforms(go("ServeHTTP")));
        assert!(!conforms(go("load_config")));
        // toolchain-mandated underscore families stay exempt — in Go
        assert!(conforms(go("ExampleParse_errors")));
        assert!(conforms(go("TestServer_Start")));
        assert!(conforms(go("Example_errors")));
        // the two dead defects: a lowercase boundary is no test name
        // (go vet's rule), and the exemption never leaves Go
        assert!(!conforms(go("Testing_helper")));
        assert!(!conforms(facts(
            Lang::TypeScript,
            NameStyle::MixedCaps,
            "TestServer_Start"
        )));
        assert!(!conforms(facts(
            Lang::Haskell,
            NameStyle::MixedCaps,
            "Testing_helper"
        )));
    }

    #[test]
    fn sentinels_pass_as_unjudged() {
        for name in ["(anonymous)", "(non-utf8)", "\"my_key\"", "[dynamic_key]"] {
            let row = facts(Lang::TypeScript, NameStyle::MixedCaps, name);
            assert_eq!(row[1], 0, "{name}: no convention applies");
            assert!(conforms(row));
        }
    }
}
