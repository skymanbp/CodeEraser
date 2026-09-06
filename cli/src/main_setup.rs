//! `ce setup`'s console body — the CLI face of the setup DOCUMENT
//! (codeeraser::setup): one measurement, rendered here or printed as
//! JSON, and the document's own exit code either way, because the
//! installer hook reads that code (the update precedent for a face
//! whose exit code is the verdict).

use crate::main_cmds::{OutFormat, json};
use codeeraser::setup;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(clap::Args)]
pub struct SetupArgs {
    /// Remove exactly what a previous `ce setup` added (keyed on its
    /// marker file); a registration you made yourself is never touched
    #[arg(long)]
    pub unwire: bool,
    /// Directory holding the `claude-plugin-wired` marker (default:
    /// this binary's directory — the installer's $INSTDIR)
    #[arg(long, value_name = "DIR")]
    pub marker_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutFormat::Console)]
    pub format: OutFormat,
}

pub fn setup_cmd(a: SetupArgs) -> ExitCode {
    let r = setup::run(&setup::Opts {
        unwire: a.unwire,
        marker_dir: a.marker_dir,
    });
    if json(a.format) {
        println!("{}", r.doc);
    } else {
        for l in setup::console(&r) {
            println!("{l}");
        }
    }
    ExitCode::from(r.exit.code())
}
