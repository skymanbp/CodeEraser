//! Per-project daemon (ADR-003): lazy start, local-socket NDJSON
//! protocol, sole SQLite writer, idle exit, version-skew restart.
//! M2 scope: ping + dedup probe; the M3 guard adds its cheap-check
//! endpoints on this same channel.

pub mod client;
mod coldstart;
pub mod proto;
pub mod server;
