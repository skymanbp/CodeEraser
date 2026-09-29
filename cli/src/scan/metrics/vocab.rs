//! The event vocabulary the complexity road speaks (plan v2.30 step
//! 7b ③; VERSIONING 7.2.0 ③): one classified node of a unit as the
//! wire carries it — `[row, seq, parent, pos, flags, aux, op…]` — and
//! the flag bits by name. The core names the same bits by position
//! in CE.Scan.Complexity, and the register the two halves replay
//! (contracts/fixtures/scan/whitepaper.ndjson) and the emitter's own
//! battery spell them in this order, so the order below IS the
//! contract rather than a convenience. Split out of the emitter when
//! that file passed the 300-line edict: what rides and who states it
//! are two jobs.

/// One classified node. `parent` is the seq of the nearest emitted
/// ancestor (None = the unit itself); `pos` is 1 in that ancestor's
/// body and 0 in its header (0 under a non-splitting ancestor); `aux`
/// is a chain's operand count; `ops` are a logic root's operator ids
/// (indices into the spec's coc_operators) in source order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub seq: u32,
    pub parent: Option<u32>,
    pub pos: u8,
    pub flags: u16,
    pub aux: u32,
    pub ops: Vec<u32>,
}

impl Event {
    /// The wire row: `[row, seq, parent, pos, flags, aux, op…]`, the
    /// parent spelled -1 for the unit itself.
    pub fn row(&self, row: u64) -> Vec<i64> {
        let head = [
            row as i64,
            i64::from(self.seq),
            self.parent.map_or(-1, i64::from),
            i64::from(self.pos),
            i64::from(self.flags),
            i64::from(self.aux),
        ];
        head.into_iter()
            .chain(self.ops.iter().map(|&o| i64::from(o)))
            .collect()
    }
}

/// One bit per name, in wire order — the first name is bit 0. Written
/// as a list rather than eleven `1 << n` lines because the order is
/// the whole content: the clone gate read the lines as copies of one
/// another, and a number typed beside each name is a second statement
/// of the same fact.
macro_rules! bits {
    ($($name:ident),* $(,)?) => { bits!(@ 0u16; $($name),*); };
    (@ $n:expr; $name:ident $(, $rest:ident)*) => {
        pub const $name: u16 = 1 << $n;
        bits!(@ $n + 1; $($rest),*);
    };
    (@ $n:expr;) => {};
}

// The class bits a node earns from the LangSpec tables — NESTING (a
// structure that raises the level), FLAT (an else / catch clause),
// NEST_ONLY (a lambda: level, no point), LABELLED_JUMP, IF_KIND, CHAIN
// (a let chain, `aux` its operand count), CC_KIND (a cyclomatic
// decision node), CC_OP (a short-circuit operator), LOGIC_ROOT (the
// head of a maximal boolean chain, `ops` its operators) — then the
// two relation bits: DIRECT (the tree parent is the parent event's
// node) and IN_ALT (the node is its tree parent's first `alternative`
// field child).
bits!(
    NESTING,
    FLAT,
    NEST_ONLY,
    LABELLED_JUMP,
    IF_KIND,
    CHAIN,
    CC_KIND,
    CC_OP,
    LOGIC_ROOT,
    DIRECT,
    IN_ALT,
);

/// Bits 11–12: the class of an if's first `alternative` child —
/// 0 none, 1 an if kind, 2 a flat kind, 3 anything else.
pub const ALT_SHIFT: u16 = 11;
