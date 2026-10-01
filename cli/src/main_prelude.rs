//! The imports every judged-family CLI face opens with (plan v2.31
//! steps 5-7): the argument helpers, the shared judge arguments, the
//! document printer and the exit code. `ce flow`, `ce query` /
//! `ce rules`, `ce similar` and `ce merge` each read these four names
//! and then their own family's modules; the four lines lived at the
//! head of every face until the clone gate read two heads as one block
//! (main_flow.rs against main_merge.rs), and one owner is the fix, not
//! a reordered import. main_judge.rs is at its own size line, so the
//! imports live here.

pub(crate) use crate::main_cmds::{fail, json, or_cwd};
pub(crate) use crate::main_judge::JudgeArgs;
pub(crate) use codeeraser::report::print_doc;
pub(crate) use std::process::ExitCode;
