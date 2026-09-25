//! A Java compilation unit's header (JLS 7.3–7.5), read lexically: the
//! package it declares and the imports it writes. Both precede every
//! type declaration, and only whitespace, comments and the package's
//! own annotations (package-info.java) may come before them, so the
//! scan stops at the first other token. The walk reads every Java file
//! this way on every run (dedup/walkidx.rs) — a tree-sitter parse per
//! file per run is what the lexer saves — and hands the headers to the
//! Java ladder (Scope::java), which never reads a file itself.

/// What the header declares: the package (`""` = the unnamed package)
/// and the imports in document order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    pub package: String,
    pub imports: Vec<Import>,
}

/// One import declaration: the dotted name as written (whitespace
/// between its tokens dropped), whether it ends `.*` and whether it is
/// `import static`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub name: String,
    pub star: bool,
    pub is_static: bool,
}

/// Read one file's header. A declaration the lexer cannot finish (no
/// closing `;`, a stray token inside it) ends the header there: what
/// was read before it stands, nothing after it is guessed.
pub fn read(text: &str) -> Header {
    let mut lex = Lexer(text.strip_prefix('\u{feff}').unwrap_or(text));
    let mut header = Header::default();
    while lex.eat('@') {
        lex.annotation();
    }
    if lex.keyword("package") {
        match lex.name() {
            Some((name, false)) if lex.eat(';') => header.package = name,
            _ => return header,
        }
    }
    while lex.keyword("import") {
        let is_static = lex.keyword("static");
        let Some((name, star)) = lex.name() else {
            break;
        };
        if !lex.eat(';') {
            break;
        }
        header.imports.push(Import {
            name,
            star,
            is_static,
        });
    }
    header
}

/// The text after an annotation whose `@` is already read — its dotted
/// name and any argument list: the Java ladder drops a type annotation
/// from the name it annotates (ladder/java.rs).
pub(super) fn past_annotation(text: &str) -> &str {
    let mut lex = Lexer(text);
    lex.annotation();
    lex.0
}

/// The unread rest of the text; every method skips the trivia before
/// the token it reads.
struct Lexer<'t>(&'t str);

impl<'t> Lexer<'t> {
    /// Whitespace and comments (an unterminated block comment runs to
    /// the end).
    fn trivia(&mut self) {
        loop {
            self.0 = self.0.trim_start();
            if let Some(rest) = self.0.strip_prefix("//") {
                self.0 = rest.split_once('\n').map_or("", |(_, tail)| tail);
            } else if let Some(rest) = self.0.strip_prefix("/*") {
                self.0 = rest.split_once("*/").map_or("", |(_, tail)| tail);
            } else {
                return;
            }
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.trivia();
        match self.0.strip_prefix(c) {
            Some(rest) => {
                self.0 = rest;
                true
            }
            None => false,
        }
    }

    /// A keyword: the word, then no identifier character.
    fn keyword(&mut self, word: &str) -> bool {
        self.trivia();
        let Some(rest) = self.0.strip_prefix(word) else {
            return false;
        };
        if rest.starts_with(ident_char) {
            return false;
        }
        self.0 = rest;
        true
    }

    fn ident(&mut self) -> Option<&'t str> {
        self.trivia();
        let end = self
            .0
            .find(|c: char| !ident_char(c))
            .unwrap_or(self.0.len());
        let (word, rest) = self.0.split_at(end);
        if word.is_empty() || word.starts_with(|c: char| c.is_ascii_digit()) {
            return None;
        }
        self.0 = rest;
        Some(word)
    }

    /// A dotted name, and whether it ended `.*`.
    fn name(&mut self) -> Option<(String, bool)> {
        let mut name = self.ident()?.to_string();
        while self.eat('.') {
            if self.eat('*') {
                return Some((name, true));
            }
            name.push('.');
            name.push_str(self.ident()?);
        }
        Some((name, false))
    }

    /// An annotation past its `@`: the (dotted) name, then an argument
    /// list if one follows.
    fn annotation(&mut self) {
        if self.name().is_some() && self.eat('(') {
            self.group();
        }
    }

    /// Past a `(` already eaten: to its matching `)`, string, text
    /// block and char literals and comments read as opaque.
    fn group(&mut self) {
        let mut depth = 1usize;
        while depth > 0 {
            self.trivia();
            let mut chars = self.0.chars();
            let Some(c) = chars.next() else {
                return;
            };
            self.0 = chars.as_str();
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                '"' | '\'' => self.literal(c),
                _ => {}
            }
        }
    }

    /// Past the opening quote of a literal: to its closing one, escapes
    /// honoured; `"""` opens a text block, closed by the next `"""`.
    fn literal(&mut self, quote: char) {
        if quote == '"'
            && let Some(rest) = self.0.strip_prefix("\"\"")
        {
            self.0 = rest.split_once("\"\"\"").map_or("", |(_, tail)| tail);
            return;
        }
        let mut chars = self.0.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                chars.next();
            } else if c == quote {
                break;
            }
        }
        self.0 = chars.as_str();
    }
}

/// A Java identifier character (JLS 3.8: letters and digits of any
/// script, `_` and `$`).
fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/java_header.rs"]
mod tests;
