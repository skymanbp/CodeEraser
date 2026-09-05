//! Term production — the ONE tokenization road index and query share
//! (spec §三: a query is hashed the way the index was, so a change
//! here moves both sides and the negative probe sees them move
//! together). Only channel-tagged fnv1a64 hashes leave this module:
//! no word text is stored downstream (plan §5.9.2 index privacy).

use super::stem::stem;
use crate::dedup::tokens::fnv1a;

/// The six evidence channels (spec §三). The one-letter label is
/// mixed into every term hash, so a name word and a callee word
/// spelled alike are two terms: a shared name is name evidence, a
/// shared callee is callee evidence, and the role rule reads them
/// apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Channel {
    Name,
    Shape,
    Callee,
    Doc,
    Structure,
    Literal,
}

impl Channel {
    /// Wire order — the six-integer evidence row `[N,P,C,D,S,L]`.
    pub const ALL: [Channel; 6] = [
        Channel::Name,
        Channel::Shape,
        Channel::Callee,
        Channel::Doc,
        Channel::Structure,
        Channel::Literal,
    ];

    /// Position in the evidence row.
    pub fn index(self) -> usize {
        Channel::ALL
            .iter()
            .position(|c| *c == self)
            .expect("every channel is listed")
    }

    /// One-letter label: the wire column head and the hash tag.
    pub fn label(self) -> &'static str {
        ["N", "P", "C", "D", "S", "L"][self.index()]
    }

    /// Query weight multiplier (spec §三 权重位): names ×3, callees ×2,
    /// everything else ×1 — integers, so the score stays exact.
    pub fn weight(self) -> u32 {
        match self {
            Channel::Name => 3,
            Channel::Callee => 2,
            _ => 1,
        }
    }

    /// Whether the channel carries WORDS (split, stemmed, PPMI-widened)
    /// rather than features the measurer spells itself.
    pub fn is_words(self) -> bool {
        matches!(self, Channel::Name | Channel::Callee | Channel::Doc)
    }
}

/// Prose stop words dropped from the doc channel — a fixed small
/// table (spec §三), never learned from a corpus. Identifier pieces
/// are NOT filtered: `get`, `set` and `is` are what a role is made of.
const STOP: [&str; 48] = [
    "a", "an", "the", "of", "to", "and", "or", "in", "on", "for", "is", "are", "be", "was", "this",
    "that", "it", "its", "as", "by", "with", "from", "at", "if", "not", "we", "you", "i", "s", "t",
    "into", "than", "then", "so", "but", "do", "does", "all", "any", "no", "our", "can", "will",
    "which", "when", "what", "each", "one",
];

/// Identifier pieces at camel / underscore / digit boundaries,
/// lowercased, empties dropped: `parseJSONFile` → parse json file,
/// `http2_server` → http 2 server, `(T) add` → t add. Any
/// non-alphanumeric character is a boundary, so prose lines split
/// through the same function (`prose_words`).
pub fn split_ident(ident: &str) -> Vec<String> {
    let chars: Vec<char> = ident.chars().collect();
    let mut out = Vec::new();
    let mut buf = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            flush(&mut buf, &mut out);
            continue;
        }
        if !buf.is_empty() && boundary(&chars, i) {
            flush(&mut buf, &mut out);
        }
        buf.extend(c.to_lowercase());
    }
    flush(&mut buf, &mut out);
    out
}

/// Whether a piece boundary falls BEFORE position `i` (i ≥ 1, both
/// sides alphanumeric): letter↔digit, lower→Upper, or the last
/// capital of a run followed by a lowercase (`JSONFile` → JSON | File).
fn boundary(chars: &[char], i: usize) -> bool {
    let (prev, c) = (chars[i - 1], chars[i]);
    if prev.is_numeric() != c.is_numeric() {
        return true;
    }
    if prev.is_lowercase() && c.is_uppercase() {
        return true;
    }
    prev.is_uppercase() && c.is_uppercase() && chars.get(i + 1).is_some_and(|n| n.is_lowercase())
}

fn flush(buf: &mut String, out: &mut Vec<String>) {
    if !buf.is_empty() {
        out.push(std::mem::take(buf));
    }
}

/// Words of one prose line: identifier-split pieces with the stop
/// words dropped, so `parseJSON` in a comment meets `parse_json` in a
/// name on the same terms.
pub fn prose_words(line: &str) -> Vec<String> {
    split_ident(line)
        .into_iter()
        .filter(|w| !STOP.contains(&w.as_str()))
        .collect()
}

/// The term of one WORD under a channel: stemmed, then hashed with
/// the channel tag.
pub fn word_term(ch: Channel, word: &str) -> u64 {
    term(ch, stem(word).as_bytes())
}

/// The term of one FEATURE (a shape, structure or literal-kind
/// spelling) under a channel — hashed as spelled, never stemmed.
pub fn feature_term(ch: Channel, feature: &[u8]) -> u64 {
    term(ch, feature)
}

fn term(ch: Channel, bytes: &[u8]) -> u64 {
    let mut v = Vec::with_capacity(bytes.len() + 2);
    v.extend_from_slice(ch.label().as_bytes());
    v.push(b':');
    v.extend_from_slice(bytes);
    fnv1a(&v)
}

#[cfg(test)]
#[path = "../../tests/unit/similar/terms.rs"]
mod tests;
