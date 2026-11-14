//! Per-project daemon (ADR-003): lazy start, local-socket NDJSON
//! protocol, idle exit, version-skew restart. What it serves is the
//! request kinds of proto.rs (`Request`); it holds the project's core
//! link across requests (judge.rs) and is one writer of the shared index
//! among others (a command run in the project writes it too).

pub mod auth;
pub mod cancel;
pub mod client;
pub mod coldstart;
pub mod judge;
pub mod proto;
pub mod server;
