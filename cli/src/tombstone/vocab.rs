//! The tombstone rule's vocabulary (plan v2.26 T track; the spec's §三
//! tables). Data only — the readers live in frames.rs and names.rs.
//! Every table is one `|`-separated string read through `entries` /
//! `has`: a table written one literal per line is a run of `LIT ,`
//! tokens, and two such tables are a clone under the dedup gate's
//! normalization (its first run on this file said so, four times).
//! Every table is a `TOMBSTONE_REV` input: extend one only with
//! corpus evidence from the FPR ledger, and bump the revision.

/// The vocabulary revision the feed records (`rev`): a reader of the
/// FPR ledger must know which tables produced a row.
pub const TOMBSTONE_REV: i64 = 1;

/// Fewest chars an ASCII name has (`ab` is a preposition, not a name).
pub const MIN_ASCII_NAME: usize = 3;
/// Fewest chars a wide (CJK) name has.
pub const MIN_WIDE_NAME: usize = 2;
/// Widest window of adjacent words one spelling covers.
pub const JOIN_MAX: usize = 3;

/// The entries of a table.
pub fn entries(table: &'static str) -> impl Iterator<Item = &'static str> {
    table.split('|')
}

/// Whether a table holds `word` exactly.
pub fn has(table: &'static str, word: &str) -> bool {
    entries(table).any(|e| e == word)
}

/// V₀ — absence words that are never names (lower-cased; the wide
/// four verbatim).
pub const NEGATIONS: &str = "no|not|non|none|null|nil|nan|noop|nop|nonzero|nonnull|notnull|\
    nostd|notfound|notimplemented|nosuch|without|unless|never|false|off|disabled|empty|void|\
    unset|missing|absent|无|非|否|空";

/// Reserved words of the six judged languages, sorted, lower-cased: a
/// name made of these alone is syntax, not a name.
pub const KEYWORDS: &str = "abstract|and|any|assert|async|await|become|boolean|box|break|\
    case|catch|chan|class|const|constructor|continue|crate|data|debugger|declare|def|\
    default|defer|del|delete|deriving|do|dyn|elif|else|enum|except|export|extends|extern|\
    fallthrough|finally|fn|for|forall|foreign|from|func|function|get|global|go|goto|hiding|\
    if|impl|implements|import|in|infix|infixl|infixr|instance|instanceof|interface|iota|is|\
    keyof|lambda|let|loop|macro|map|match|mod|module|move|mut|namespace|new|newtype|\
    nonlocal|number|object|of|or|package|pass|private|protected|pub|public|qualified|raise|\
    range|readonly|ref|require|return|select|self|set|static|string|struct|super|switch|\
    symbol|then|this|throw|trait|true|try|type|typeof|undefined|union|unique|unknown|unsafe|\
    use|var|where|while|with|yield";

/// English absence frames: a prefix binds the words after it, a
/// suffix the words before it.
pub const EN_PREFIX: &str = "no|not|non|without|sans|minus";
pub const EN_SUFFIX: &str = "free|less|removed|dropped|gone";

/// Chinese absence frames, read inside one run (`无东坡肉`) or as a
/// word before an ASCII name (`无cache`).
pub const ZH_PREFIX: &str = "无|非|不含|不带|没有|去掉|去|免|已删";
pub const ZH_SUFFIX: &str = "已删|已移除|已去掉|不再有";

/// Retrospective marks a prose segment carries: lower-cased English
/// phrases matched at word boundaries, Chinese by substring.
pub const MARKS_EN: &str = "no longer|previously|formerly|used to|we removed|was removed|\
    has been removed|is no longer needed|is not needed here|is unnecessary|\
    there is no need to|deliberately omitted|intentionally absent|rather than adding";
pub const MARKS_ZH: &str = "不再|此前|原先|曾经|已去掉|已删除|已移除|不需要|没有必要|无需|\
    故意不|刻意不加|之所以不";

/// English function words: a window holding one straddles a phrase
/// boundary (`the_pre`, `budget_is`, `self_and_nth` — sentence-shaped
/// test names cut into windows) and is no name. Read only through
/// `vocabulary` (and the test that walks every table).
const STOP_EN: &str = "a|an|the|is|are|was|were|be|been|being|am|of|to|in|on|at|by|for|\
    with|as|it|its|this|that|these|those|than|then|and|or|but|so|if|do|does|did|has|have|\
    had|from|into|onto|over|under|up|down|out|off|all|any|each|per|via|vs|we|you|they|\
    our|your|their|my|me|us|him|her|his|who|what|which|when|where|why|how|here|there|\
    also|only|just|very|still|yet|too|both|either|neither|such|same|other|another|\
    should|would|could|can|may|might|must|will|shall";

/// Whether `w` is a word of the instrument's own vocabulary — a frame,
/// an absence word, a function word, or a word of a retrospective
/// mark: none of these spells a name, on either side of a change.
pub fn vocabulary(w: &str) -> bool {
    [
        NEGATIONS, EN_PREFIX, EN_SUFFIX, ZH_PREFIX, ZH_SUFFIX, STOP_EN,
    ]
    .iter()
    .any(|t| has(t, w))
        || entries(MARKS_EN).any(|phrase| phrase.split(' ').any(|x| x == w))
}

/// Bracket pairs (ASCII and full-width) that make a frame `bracketed`.
pub const OPEN: &[char] = &['(', '（', '[', '【'];
pub const CLOSE: &[char] = &[')', '）', ']', '】'];

#[cfg(test)]
#[path = "../../tests/unit/tombstone/vocab.rs"]
mod tests;
