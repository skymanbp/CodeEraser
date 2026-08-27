//! codeeraser — library surface. The `ce` binary is a thin CLI over
//! this; the M2 daemon and integration tests consume it directly.

pub mod allow;
pub mod audit;
pub mod churn;
pub mod config;
pub mod corelink;
pub mod daemon;
pub mod dedup;
pub mod docdup;
pub mod eject;
pub mod erase;
pub mod faces;
pub mod fourclass;
pub mod graph;
pub mod guard;
pub mod health;
pub mod hookio;
pub mod i18n;
pub mod join;
pub mod lockstep;
pub mod mcp;
pub mod mention;
pub mod proc;
pub mod progress;
pub mod report;
pub mod root;
pub mod sarif;
pub mod scan;
pub mod score;
pub mod structure;
pub mod trend;

#[cfg(test)]
pub(crate) mod testutil;
