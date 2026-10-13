//! The flow family's hub command (plan v2.31 step 5) — the SAME
//! document `ce flow --format json` prints: the dead code inside every
//! function of the tree, judged by the core's flow/1 and placed through
//! the lowering's legend; `kind` narrows the listed findings as
//! `--kind` does (absent = every kind; the hub's kind chips ask with
//! the kinds they show, and the counts stay whole). Rendering is the webview's
//! (hub_flow.js), and a core without the family shows the document's
//! named degraded reason.

use crate::commands::face_cmd;
use codeeraser::faces::{self, doc, Plumb};

face_cmd!(flow_report, "flow", (kind: Option<Vec<String>>) =>
    |r, c, k: Option<Vec<String>>| doc(faces::flow(r, c, &k.unwrap_or_default(), Plumb::default())));

// The flow kinds as the core's catalogue lists them, off the definition
// package the task loads (faces::flow_kinds).
face_cmd!(flow_kinds, "flow_kinds", |_, _| Ok(faces::flow_kinds()));
