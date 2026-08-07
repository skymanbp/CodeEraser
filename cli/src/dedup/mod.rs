//! `ce dedup` clone-detection hot path (plan ADR-005): normalized
//! token stream → winnowing/Rabin-Karp fingerprints (Schleimer et al.
//! SIGMOD'03) → inverted index. T1/T2 only here; T3 is the M5 cold
//! path. This module is pure (no I/O) — the SQLite index and daemon
//! layers consume it.

pub mod tokens;
pub mod winnow;

/// Winnowing parameters. Guarantee threshold t = matches of at least
/// `t` normalized tokens are always detected (SIGMOD'03 correctness
/// bound); noise threshold k = matches shorter than `kgram` tokens are
/// never reported. window = t - k + 1.
#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub kgram: usize,
    pub window: usize,
}

impl Default for Params {
    /// t = 50 tokens aligns with the jscpd min-tokens default
    /// (plan §4.1 clone row); k = 25 → window 26.
    fn default() -> Self {
        Self {
            kgram: 25,
            window: 26,
        }
    }
}
