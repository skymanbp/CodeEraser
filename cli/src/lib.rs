//! codeeraser — library surface. The `ce` binary is a thin CLI over
//! this; the M2 daemon and integration tests consume it directly.

pub mod allow;
pub mod arch;
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
pub mod flow;
pub mod flow_report;
pub mod fourclass;
pub mod gitmodules;
pub mod graph;
pub mod guard;
pub mod health;
pub mod hookio;
pub mod i18n;
pub mod join;
pub mod lockstep;
pub mod mcp;
pub mod mention;
pub mod merge;
pub mod proc;
pub mod progress;
pub mod query;
pub mod report;
pub mod root;
pub mod sarif;
pub mod scan;
pub mod score;
pub mod setup;
pub mod similar;
pub mod structure;
pub mod tables;
pub mod tombstone;
pub mod trend;
pub mod update;

#[cfg(test)]
#[path = "../tests/unit/testutil.rs"]
pub(crate) mod testutil;
