//! codeeraser — library surface. The `ce` binary is a thin CLI over
//! this; the M2 daemon and integration tests consume it directly.

pub mod config;
pub mod dedup;
pub mod handshake;
pub mod scan;
