//! The six evidence channels of a term. The term road itself — one
//! tokenization for index and query (spec §三) — is the core's since
//! plan v2.33 W6 (CE.Similar.Bags over bags/1, bags.rs): only
//! channel-tagged fnv1a64 hashes come back, and no word text is stored
//! downstream (plan §5.9.2 index privacy).

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

    /// Whether the channel carries WORDS (split, stemmed, PPMI-widened)
    /// rather than features the measurer spells itself.
    pub fn is_words(self) -> bool {
        matches!(self, Channel::Name | Channel::Callee | Channel::Doc)
    }
}
