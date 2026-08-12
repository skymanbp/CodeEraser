//! codeeraser — library surface. The `ce` binary is a thin CLI over
//! this; the M2 daemon and integration tests consume it directly.

pub mod audit;
pub mod churn;
pub mod config;
pub mod corelink;
pub mod daemon;
pub mod dedup;
pub mod fourclass;
pub mod graph;
pub mod guard;
pub mod health;
pub mod hookio;
pub mod mcp;
pub mod scan;
