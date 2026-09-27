//! The type declarations of a Java compilation unit, read lexically —
//! java_header.rs's child (plan v2.30 step 5b): every `class`,
//! `interface`, `enum`, `record` and `@interface` with its simple name,
//! the supertypes its header writes (`extends` and `implements`, type
//! arguments and annotations dropped, `permits` and beyond skipped),
//! its member types and the lines it spans. The scan tracks braces: a
//! body opened by a type keyword is a type frame, every other brace a
//! block, and a type closing inside a block — a local class in a method
//! body, an initializer, an anonymous class body, an enum constant's
//! body — is no member of anything and is dropped. Literals and
//! comments are skipped as the header lexer skips them, `Foo.class` is
//! no keyword, and `record` is one only before a name and its header's
//! `(` or `<` (JLS 3.9: a contextual keyword).

/// One type declaration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeDecl {
    pub name: String,
    /// The supertypes as written, dotted names kept (`a.b.C`,
    /// `Outer.Inner`), in header order.
    pub supers: Vec<String>,
    pub members: Vec<TypeDecl>,
    /// First and last line of the declaration, 1-based, inclusive.
    pub lines: (usize, usize),
}

enum Frame {
    Block,
    Type(TypeDecl),
}

/// The unit's top-level types, member types nested inside them.
pub fn read_types(text: &str) -> Vec<TypeDecl> {
    let mut scan = Scan {
        text,
        pos: 0,
        line: 1,
        tok_line: 1,
        last_dot: false,
        after_dot: false,
    };
    let (mut stack, mut out) = (Vec::<Frame>::new(), Vec::new());
    while let Some(tok) = scan.next() {
        match tok {
            Tok::Word(word) if scan.opens_type(word) => {
                if let Some(decl) = scan.declaration() {
                    stack.push(Frame::Type(decl));
                }
            }
            Tok::Punct('{') => stack.push(Frame::Block),
            Tok::Punct('}') => close(&mut stack, &mut out, scan.tok_line),
            _ => {}
        }
    }
    out
}

/// A closing brace: a type frame ends on this line and joins its
/// enclosing type's members, the unit's types, or — inside a block —
/// nothing.
fn close(stack: &mut Vec<Frame>, out: &mut Vec<TypeDecl>, line: usize) {
    let Some(Frame::Type(mut decl)) = stack.pop() else {
        return;
    };
    decl.lines.1 = line;
    match stack.last_mut() {
        Some(Frame::Type(parent)) => parent.members.push(decl),
        Some(Frame::Block) => {}
        None => out.push(decl),
    }
}

#[derive(Clone, Copy)]
enum Tok<'t> {
    Word(&'t str),
    Punct(char),
}

/// The scan's position: the text, the byte offset, the line there, the
/// line of the token last read, and whether that token — and the one
/// before it — was a `.` (`Foo.class` is no keyword).
#[derive(Clone)]
struct Scan<'t> {
    text: &'t str,
    pos: usize,
    line: usize,
    tok_line: usize,
    last_dot: bool,
    after_dot: bool,
}

impl<'t> Scan<'t> {
    /// The next token past trivia and literals: a word (identifier,
    /// keyword or number) or one punctuation character.
    fn next(&mut self) -> Option<Tok<'t>> {
        loop {
            self.trivia();
            let rest = &self.text[self.pos..];
            let c = rest.chars().next()?;
            self.tok_line = self.line;
            if c == '"' || c == '\'' {
                self.advance(c.len_utf8());
                self.literal(c);
                continue;
            }
            let tok = if ident_char(c) {
                let end = rest.find(|ch: char| !ident_char(ch)).unwrap_or(rest.len());
                self.advance(end);
                Tok::Word(&rest[..end])
            } else {
                self.advance(c.len_utf8());
                Tok::Punct(c)
            };
            self.after_dot = self.last_dot;
            self.last_dot = matches!(tok, Tok::Punct('.'));
            return Some(tok);
        }
    }

    fn peek(&self) -> Option<Tok<'t>> {
        self.clone().next()
    }

    fn advance(&mut self, bytes: usize) {
        self.line += self.text[self.pos..self.pos + bytes].matches('\n').count();
        self.pos += bytes;
    }

    /// Whitespace and comments (an unterminated block comment runs to
    /// the end).
    fn trivia(&mut self) {
        loop {
            let rest = &self.text[self.pos..];
            self.advance(rest.len() - rest.trim_start().len());
            let rest = &self.text[self.pos..];
            if rest.starts_with("//") {
                self.advance(rest.find('\n').unwrap_or(rest.len()));
            } else if let Some(body) = rest.strip_prefix("/*") {
                self.advance(body.find("*/").map_or(rest.len(), |i| i + 4));
            } else {
                return;
            }
        }
    }

    /// Past the opening quote: to the closing one, escapes honoured,
    /// an unterminated literal ending at its line (javac's own error);
    /// `"""` opens a text block, closed by the next `"""`.
    fn literal(&mut self, quote: char) {
        let rest = &self.text[self.pos..];
        if quote == '"'
            && let Some(block) = rest.strip_prefix("\"\"")
        {
            self.advance(block.find("\"\"\"").map_or(rest.len(), |i| i + 5));
            return;
        }
        let mut chars = rest.char_indices();
        while let Some((i, c)) = chars.next() {
            if c == '\\' {
                chars.next();
            } else if c == quote || c == '\n' {
                self.advance(i + c.len_utf8());
                return;
            }
        }
        self.advance(rest.len());
    }

    /// Whether the word just read opens a type declaration.
    fn opens_type(&self, word: &str) -> bool {
        if self.after_dot {
            return false;
        }
        match word {
            "class" | "interface" | "enum" => true,
            "record" => {
                let mut probe = self.clone();
                matches!(probe.next(), Some(Tok::Word(_)))
                    && matches!(probe.next(), Some(Tok::Punct('(' | '<')))
            }
            _ => false,
        }
    }

    /// The rest of a declaration's header past its keyword: the name,
    /// then to the opening brace — type parameters and a record header
    /// skipped whole, an annotation's name skipped, the names after
    /// `extends` / `implements` collected, `permits` and beyond not.
    /// None = no body opens (the text ends, or a `;` comes first).
    fn declaration(&mut self) -> Option<TypeDecl> {
        let start = self.tok_line;
        let Some(Tok::Word(name)) = self.next() else {
            return None;
        };
        let (mut supers, mut collecting) = (Vec::new(), false);
        loop {
            match self.next()? {
                Tok::Punct('{') => break,
                Tok::Punct(';') => return None,
                Tok::Punct('<') => self.skip_group('<', '>'),
                Tok::Punct('(') => self.skip_group('(', ')'),
                Tok::Punct('@') => self.annotation_name(),
                Tok::Word("extends" | "implements") => collecting = true,
                Tok::Word("permits") => collecting = false,
                Tok::Word(word) if collecting => supers.push(self.dotted(word)),
                _ => {}
            }
        }
        Some(TypeDecl {
            name: name.to_string(),
            supers,
            members: Vec::new(),
            lines: (start, 0),
        })
    }

    /// A type name from its first word on: a `.` and a word continue
    /// it, a type argument list between is skipped (`Outer<T>.Inner`).
    fn dotted(&mut self, first: &str) -> String {
        let mut name = first.to_string();
        loop {
            match self.peek() {
                Some(Tok::Punct('<')) => {
                    self.next();
                    self.skip_group('<', '>');
                }
                Some(Tok::Punct('.')) => {
                    let mut probe = self.clone();
                    probe.next();
                    let Some(Tok::Word(word)) = probe.next() else {
                        break;
                    };
                    name.push('.');
                    name.push_str(word);
                    *self = probe;
                }
                _ => break,
            }
        }
        name
    }

    /// An annotation's (dotted) name past its `@`; its argument list,
    /// if any, is the header loop's `(` arm.
    fn annotation_name(&mut self) {
        if let Some(Tok::Word(word)) = self.peek() {
            self.next();
            self.dotted(word);
        }
    }

    /// Past an opening bracket already read: to its match.
    fn skip_group(&mut self, open: char, close: char) {
        let mut depth = 1usize;
        while depth > 0 {
            match self.next() {
                None => return,
                Some(Tok::Punct(c)) if c == open => depth += 1,
                Some(Tok::Punct(c)) if c == close => depth -= 1,
                _ => {}
            }
        }
    }
}

/// A Java identifier character (JLS 3.8: letters and digits of any
/// script, `_` and `$`) — the header lexer imports it from here so the
/// edge between the two files runs one way.
pub(super) fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

#[cfg(test)]
#[path = "../../../tests/unit/graph/ladder/java_types.rs"]
mod tests;
