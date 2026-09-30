//! The query language's scanner (design booklet §4.4): the text of a
//! program into the `[kind, value]` tokens the core parses, each
//! keeping the line and column it came from, so an error the core
//! reports at a token index reads back as `where line:column`. The
//! scanner knows spellings and nothing of shapes: variables number
//! per clause by first appearance (`_` takes a fresh number), program
//! predicates number from 1000 by first appearance across every
//! source, a lowercase word is a predicate when a `(` follows it and a
//! name constant otherwise, a quoted string is a name except in the
//! one position whose sort is a set — `set(S, …)`'s first argument
//! and the `in(F, "glob")` sugar, rewritten here to `set(S, F)`.

use super::legend;
use super::program::{Fault, Program, Source, Token};
use std::collections::HashMap;
use std::sync::OnceLock;

/// The value-bearing token kinds (CE.Query.Cost; 5 is unassigned);
/// punctuation and keywords are the codes `punct` answers, 10 up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Pred,
    Var,
    Int,
    Set,
    Sym,
    Anon,
}

impl Kind {
    pub fn code(self) -> i64 {
        match self {
            Kind::Pred => 0,
            Kind::Var => 1,
            Kind::Int => 2,
            Kind::Set => 3,
            Kind::Sym => 4,
            Kind::Anon => 6,
        }
    }
}

/// Punctuation and keywords by spelling (CE.Query.Cost's codes).
const PUNCT: &str = ":- 10\n, 11\n. 12\n( 13\n) 14\nnot 15\n?- 16\nassert 17\n= 18\n!= 19\n\
< 20\n<= 21\n> 22\n>= 23\n+ 24\n- 25\n* 26\n/ 27\n% 28\ncount 29\nmin 30\nmax 31\nsum 32\n: 33";

/// The code of a punctuation or keyword spelling, if it is one.
pub fn punct(spell: &str) -> Option<i64> {
    static M: OnceLock<HashMap<&'static str, i64>> = OnceLock::new();
    M.get_or_init(|| {
        PUNCT
            .lines()
            .filter_map(|l| l.split_once(' '))
            .map(|(s, c)| (s, c.parse().expect("punct code")))
            .collect()
    })
    .get(spell)
    .copied()
}

/// One open parenthesis: the predicate it opened, when it opened an
/// atom, and the argument the scanner is inside.
struct Frame {
    pred: Option<String>,
    arg: usize,
}

pub(super) struct Lexer<'a> {
    p: &'a mut Program,
    src: Source,
    b: &'a [u8],
    i: usize,
    line: u32,
    col: u32,
    vars: Vec<String>,
    frames: Vec<Frame>,
    pending_pred: Option<String>,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(p: &'a mut Program, src: Source, text: &'a str) -> Self {
        Lexer {
            p,
            src,
            b: text.as_bytes(),
            i: 0,
            line: 1,
            col: 1,
            vars: Vec::new(),
            frames: Vec::new(),
            pending_pred: None,
        }
    }

    /// The whole source; an unfinished clause at the end still seats
    /// its variable table (the core names the syntax error there).
    pub(super) fn run(mut self) -> Result<(), Fault> {
        loop {
            self.skip_space();
            if self.i >= self.b.len() {
                break;
            }
            self.item()?;
        }
        if self.open_clause() {
            self.end_clause();
        }
        Ok(())
    }

    fn open_clause(&self) -> bool {
        self.p
            .tokens
            .last()
            .is_some_and(|t| t.src == self.src && !(t.spell == "." && self.frames.is_empty()))
    }

    /// A fault where the scanner stands.
    fn fault(&self, what: &str) -> Fault {
        self.fault_at(self.line, self.col, what)
    }

    /// A fault pinned where the offending item began.
    fn fault_at(&self, line: u32, col: u32, what: &str) -> Fault {
        Fault {
            src: self.src,
            line,
            col,
            what: what.to_string(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn bump(&mut self) -> u8 {
        let c = self.b[self.i];
        self.i += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else if c & 0xC0 != 0x80 {
            self.col += 1;
        }
        c
    }

    fn skip_space(&mut self) {
        while let Some(c) = self.peek() {
            if c == b'#' {
                while self.peek().is_some_and(|c| c != b'\n') {
                    self.bump();
                }
            } else if c.is_ascii_whitespace() {
                self.bump();
            } else {
                return;
            }
        }
    }

    /// The byte after any whitespace, without moving.
    fn next_solid(&self) -> Option<u8> {
        self.b[self.i..]
            .iter()
            .copied()
            .find(|c| !c.is_ascii_whitespace())
    }

    fn push(&mut self, kind: i64, value: i128, spell: String, line: u32, col: u32) {
        self.p.tokens.push(Token {
            kind,
            value,
            src: self.src,
            line,
            col,
            spell,
        });
    }

    fn item(&mut self) -> Result<(), Fault> {
        let c = self.peek().expect("an item byte");
        let (line, col) = (self.line, self.col);
        let digit_next = self.b.get(self.i + 1).is_some_and(u8::is_ascii_digit);
        match c {
            b'"' => self.string(line, col),
            b'0'..=b'9' => self.number(false, line, col),
            b'-' if digit_next && !self.operand_before() => self.number(true, line, col),
            c if c.is_ascii_alphabetic() || c == b'_' => self.word_token(line, col),
            _ => self.punct_token(line, col),
        }
    }

    /// Whether the last token ends an operand — then a `-` is the
    /// operator, not a negative literal's sign.
    fn operand_before(&self) -> bool {
        self.p
            .tokens
            .last()
            .is_some_and(|t| (1..=6).contains(&t.kind) || t.spell == ")")
    }

    fn word(&mut self) -> String {
        let start = self.i;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            self.bump();
        }
        String::from_utf8_lossy(&self.b[start..self.i]).into_owned()
    }

    /// `"…"` up to the closing quote on the same line: a set in the
    /// set position, a name anywhere else.
    fn string(&mut self, line: u32, col: u32) -> Result<(), Fault> {
        self.bump();
        let start = self.i;
        while self.peek().is_some_and(|c| c != b'"' && c != b'\n') {
            self.bump();
        }
        if self.peek() != Some(b'"') {
            return Err(self.fault_at(line, col, "unterminated string"));
        }
        let text = String::from_utf8_lossy(&self.b[start..self.i]).into_owned();
        self.bump();
        if self.in_set_position() {
            let id = self.p.set_id(&text);
            self.push(Kind::Set.code(), id, text, line, col);
        } else {
            self.name(text, line, col);
        }
        Ok(())
    }

    fn in_set_position(&self) -> bool {
        self.frames.last().is_some_and(|f| {
            f.pred.as_deref() == Some(legend::SET_PRED) && f.arg == legend::SET_ARG
        })
    }

    fn name(&mut self, text: String, line: u32, col: u32) {
        let hash = legend::sym(&text);
        self.p.names.entry(hash).or_insert_with(|| text.clone());
        self.push(Kind::Sym.code(), i128::from(hash), text, line, col);
    }

    fn number(&mut self, negative: bool, line: u32, col: u32) -> Result<(), Fault> {
        let start = self.i;
        if negative {
            self.bump();
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.bump();
        }
        let spell = String::from_utf8_lossy(&self.b[start..self.i]).into_owned();
        let digits = spell.trim_start_matches('-');
        let value = match (digits.parse::<u64>(), negative) {
            (Ok(n), false) => i128::from(n),
            (Ok(n), true) if n <= 1u64 << 63 => -i128::from(n),
            _ => return Err(self.fault_at(line, col, "integer out of range")),
        };
        self.push(Kind::Int.code(), value, spell, line, col);
        Ok(())
    }

    fn word_token(&mut self, line: u32, col: u32) -> Result<(), Fault> {
        let w = self.word();
        let first = w.as_bytes()[0];
        if first.is_ascii_uppercase() || first == b'_' {
            self.variable(w, line, col);
        } else if let Some(code) = punct(&w) {
            self.push(code, 0, w, line, col);
        } else if self.next_solid() == Some(b'(') {
            if w == "in" {
                return self.sugar(line, col);
            }
            let code = match legend::pred(&w) {
                Some(p) => i128::from(p.code),
                None => i128::from(self.p.idb_code(&w)),
            };
            self.pending_pred = Some(w.clone());
            self.push(Kind::Pred.code(), code, w, line, col);
        } else {
            self.name(w, line, col);
        }
        Ok(())
    }

    /// A variable by first appearance in its clause; `_` (and any
    /// word starting with it) takes a fresh number every time.
    fn variable(&mut self, w: String, line: u32, col: u32) {
        let anon = w.as_bytes()[0] == b'_';
        let seat = (!anon)
            .then(|| self.vars.iter().position(|v| *v == w))
            .flatten();
        let n = seat.unwrap_or_else(|| {
            self.vars.push(if anon { "_".into() } else { w.clone() });
            self.vars.len() - 1
        });
        let kind = if anon { Kind::Anon } else { Kind::Var };
        self.push(kind.code(), n as i128, w, line, col);
    }

    fn punct_token(&mut self, line: u32, col: u32) -> Result<(), Fault> {
        let end = (self.i + 2).min(self.b.len());
        let two = String::from_utf8_lossy(&self.b[self.i..end]).into_owned();
        let spell = if two.len() == 2 && punct(&two).is_some() {
            two
        } else {
            String::from_utf8_lossy(&self.b[self.i..self.i + 1]).into_owned()
        };
        let Some(code) = punct(&spell) else {
            return Err(self.fault("unexpected character"));
        };
        for _ in 0..spell.len() {
            self.bump();
        }
        match spell.as_str() {
            "(" => self.frames.push(Frame {
                pred: self.pending_pred.take(),
                arg: 0,
            }),
            ")" if self.frames.pop().is_none() => {
                return Err(self.fault_at(line, col, "unbalanced `)`"));
            }
            "," => {
                if let Some(f) = self.frames.last_mut() {
                    f.arg += 1;
                }
            }
            _ => {}
        }
        self.push(code, 0, spell.clone(), line, col);
        if spell == "." && self.frames.is_empty() {
            self.end_clause();
        }
        Ok(())
    }

    fn end_clause(&mut self) {
        self.p.vars.push(std::mem::take(&mut self.vars));
        self.p.clauses += 1;
    }

    /// `in(F, "glob")` → `set(S, F)`: the tokens the spelled-out form
    /// carries, the glob and the file term swapped into set order.
    fn sugar(&mut self, line: u32, col: u32) -> Result<(), Fault> {
        const SHAPE: &str = "in(File, \"glob\") takes a variable and a quoted glob";
        let set_code = i128::from(legend::pred(legend::SET_PRED).expect("set").code);
        self.push(Kind::Pred.code(), set_code, "in".into(), line, col);
        self.expect(b'(', SHAPE)?;
        self.skip_space();
        let (tl, tc) = (self.line, self.col);
        let term = match self.peek() {
            Some(c) if c.is_ascii_uppercase() || c == b'_' => self.word(),
            _ => return Err(self.fault(SHAPE)),
        };
        self.expect(b',', SHAPE)?;
        self.skip_space();
        let (gl, gc) = (self.line, self.col);
        if self.peek() != Some(b'"') {
            return Err(self.fault(SHAPE));
        }
        self.frames.push(Frame {
            pred: Some(legend::SET_PRED.into()),
            arg: legend::SET_ARG,
        });
        self.string(gl, gc)?;
        self.frames.pop();
        let comma = self.p.tokens.len() - 1;
        self.p.tokens.swap(comma - 1, comma);
        self.variable(term, tl, tc);
        self.expect(b')', SHAPE)
    }

    /// One punctuation byte after any whitespace, or the shape fault.
    fn expect(&mut self, c: u8, shape: &str) -> Result<(), Fault> {
        self.skip_space();
        if self.peek() != Some(c) {
            return Err(self.fault(shape));
        }
        let (line, col) = (self.line, self.col);
        self.bump();
        let spell = String::from_utf8_lossy(&[c]).into_owned();
        self.push(punct(&spell).expect("punct"), 0, spell, line, col);
        Ok(())
    }
}
