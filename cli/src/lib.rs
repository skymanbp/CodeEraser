//! codeeraser — library surface. The `ce` binary is a thin CLI over
//! this; the M2 daemon and integration tests consume it directly.

pub mod audit;
pub mod config;
pub mod daemon;
pub mod dedup;
pub mod guard;
pub mod handshake;
pub mod scan;
