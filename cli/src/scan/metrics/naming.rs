//! Readability naming check (plan §4.1): per-language function-name
//! convention. Deliberately coarse — it flags convention *violations*
//! (wrong case family), not style nits; provenance: PEP 8 (Python),
//! RFC 430 (Rust), Effective Go, common TS practice.

use crate::scan::spec::NameStyle;

/// Go test-function families where godoc REQUIRES underscores
/// (Example_suffix, TestType_Method subtables etc.).
const GO_TEST_PREFIXES: &[&str] = &["Test", "Benchmark", "Example", "Fuzz"];

/// Sentinel names like "(anonymous)" and non-identifier subjects
/// (quoted / computed TS object keys) are not judged. Leading
/// underscores (unused/private markers) are tolerated everywhere.
pub fn conforms(style: NameStyle, name: &str) -> bool {
    if name.starts_with(['(', '"', '\'', '[']) {
        return true;
    }
    let core = name.trim_start_matches('_');
    match style {
        NameStyle::Any => true,
        NameStyle::Snake => !core.contains(char::is_uppercase),
        NameStyle::MixedCaps => {
            !core.contains('_') || GO_TEST_PREFIXES.iter().any(|p| core.starts_with(p))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_rules() {
        assert!(conforms(NameStyle::Snake, "load_config"));
        assert!(conforms(NameStyle::Snake, "__init__"));
        assert!(!conforms(NameStyle::Snake, "loadConfig"));
    }

    #[test]
    fn mixed_caps_rules() {
        assert!(conforms(NameStyle::MixedCaps, "loadConfig"));
        assert!(conforms(NameStyle::MixedCaps, "ServeHTTP"));
        assert!(!conforms(NameStyle::MixedCaps, "load_config"));
        // godoc-mandated underscore families must not be flagged
        assert!(conforms(NameStyle::MixedCaps, "ExampleParse_errors"));
        assert!(conforms(NameStyle::MixedCaps, "TestServer_Start"));
    }

    #[test]
    fn sentinels_pass() {
        assert!(conforms(NameStyle::Snake, "(anonymous)"));
        assert!(conforms(NameStyle::MixedCaps, "(non-utf8)"));
        // quoted / computed TS object keys are not identifiers
        assert!(conforms(NameStyle::MixedCaps, "\"my_key\""));
        assert!(conforms(NameStyle::MixedCaps, "[dynamic_key]"));
    }
}
