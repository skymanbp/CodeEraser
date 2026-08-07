//! Language identification and per-language tree-sitter grammar lookup.
//! M1 launch set (plan §6 M1): TypeScript / Python / Rust / Go / Markdown
//! (Markdown is size-only: no grammar, no functions).

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Python,
    TypeScript,
    Tsx,
    Rust,
    Go,
    Markdown,
}

impl Lang {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "py" => Some(Self::Python),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }

    /// Grammar for AST-backed languages; None = size-only (Markdown).
    pub fn grammar(self) -> Option<tree_sitter::Language> {
        match self {
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Self::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::Go => Some(tree_sitter_go::LANGUAGE.into()),
            Self::Markdown => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Markdown => "markdown",
        }
    }
}
