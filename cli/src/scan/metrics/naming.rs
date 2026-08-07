//! Readability naming check (plan §4.1): per-language function-name
//! convention. Deliberately coarse — it flags convention *violations*
//! (wrong case family), not style nits; provenance: PEP 8 (Python),
//! RFC 430 (Rust), Effective Go, common TS practice.

use crate::scan::spec::NameStyle;

/// Sentinel names like "(anonymous)" are not identifiers — always pass.
/// Leading underscores (unused/private markers) are tolerated everywhere.
pub fn conforms(style: NameStyle, name: &str) -> bool {
    if name.starts_with('(') {
        return true;
    }
    let core = name.trim_start_matches('_');
    match style {
        NameStyle::Any => true,
        NameStyle::Snake => !core.contains(char::is_uppercase),
        NameStyle::MixedCaps => !core.contains('_'),
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
    }

    #[test]
    fn sentinels_pass() {
        assert!(conforms(NameStyle::Snake, "(anonymous)"));
        assert!(conforms(NameStyle::MixedCaps, "(non-utf8)"));
    }
}
