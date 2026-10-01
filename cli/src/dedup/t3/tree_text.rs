//! The two token columns merge/1 reads beside each tree node (plan
//! v2.31 steps 6-7, merge generation 2 ruling R1; design booklet §6.2):
//! `own`, the fnv1a64 of the node's own anonymous tokens — its direct
//! children that are not named, looking through anonymous
//! intermediates to their tokens and never into a named child — and
//! `text`, the fnv1a64 of its subtree's whole token stream, the named
//! leaves' tokens and the anonymous ones in source order. Both join
//! their tokens' bytes with one 0x00 between two tokens; no token at all
//! hashes to 0. A token is a node with no children (a named leaf like an
//! identifier, or punctuation); without extras a comment is no token,
//! the tree's own rule (tree.rs). A one-token leaf's `text` equals its
//! `leaf` — a coincidence, not a contract: `leaf` hashes a node's whole
//! source bytes, so `()` and `( )` differ there and agree here.

use super::tree::{Extras, kids, skipped};
use crate::dedup::tokens::fnv1a;
use crate::scan::ast;
use std::collections::HashMap;
use tree_sitter::Node;

/// One top node's tokens in source order and, per node of its subtree,
/// the run of them it spans.
pub struct Tokens<'a> {
    bytes: &'a [u8],
    tokens: Vec<(usize, usize)>,
    runs: HashMap<usize, (usize, usize)>,
    extras: Extras,
}

impl<'a> Tokens<'a> {
    /// One walk of `top`'s whole subtree, named and anonymous (an
    /// explicit stack, like every walker in this crate).
    pub fn of(top: Node, text: &'a str, extras: Extras) -> Self {
        let mut t = Tokens {
            bytes: text.as_bytes(),
            tokens: Vec::new(),
            runs: HashMap::new(),
            extras,
        };
        let mut stack = vec![(top, None)];
        while let Some((n, entered)) = stack.pop() {
            if let Some(start) = entered {
                t.runs.insert(n.id(), (start, t.tokens.len()));
            } else if !skipped(n, t.extras) {
                stack.push((n, Some(t.tokens.len())));
                if n.child_count() == 0 {
                    t.tokens.push((n.start_byte(), n.end_byte()));
                }
                stack.extend(ast::children(n).into_iter().rev().map(|c| (c, None)));
            }
        }
        t
    }

    /// The hash of the node's whole token stream (0 for a node this
    /// walk did not reach).
    pub fn text(&self, node: Node) -> u64 {
        self.runs
            .get(&node.id())
            .map_or(0, |&(s, e)| self.hash(&self.tokens[s..e]))
    }

    /// The hash of the node's own anonymous tokens — the tokens the
    /// tree's own child walk steps over (tree.rs `kids`).
    pub fn own(&self, node: Node) -> u64 {
        let (_, own) = kids(node, self.extras);
        let own: Vec<(usize, usize)> = own.iter().map(|c| (c.start_byte(), c.end_byte())).collect();
        self.hash(&own)
    }

    /// The node's token stream as bytes, 0x00 between two tokens — what
    /// a run of top nodes joins into its synthetic root's text.
    pub fn stream(&self, node: Node) -> Vec<u8> {
        self.runs
            .get(&node.id())
            .map_or_else(Vec::new, |&(s, e)| self.joined(&self.tokens[s..e]))
    }

    fn joined(&self, tokens: &[(usize, usize)]) -> Vec<u8> {
        join(tokens.iter().map(|&(s, e)| &self.bytes[s..e]))
    }

    fn hash(&self, tokens: &[(usize, usize)]) -> u64 {
        if tokens.is_empty() {
            0
        } else {
            fnv1a(&self.joined(tokens))
        }
    }
}

/// The text of a run of token streams joined end to end (a fragment's
/// synthetic root over its top nodes), 0x00 between two of them.
pub fn joined_text(streams: &[&[u8]]) -> u64 {
    if streams.is_empty() {
        0
    } else {
        fnv1a(&join(streams.iter().copied()))
    }
}

/// Byte runs joined with one 0x00 between two of them.
fn join<'b>(parts: impl Iterator<Item = &'b [u8]>) -> Vec<u8> {
    let mut out = Vec::new();
    for (k, part) in parts.enumerate() {
        if k > 0 {
            out.push(0);
        }
        out.extend_from_slice(part);
    }
    out
}

#[cfg(test)]
#[path = "../../../tests/unit/dedup/t3/tree_text.rs"]
mod tests;
