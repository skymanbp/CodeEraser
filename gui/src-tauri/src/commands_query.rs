//! The Query screen's two commands (plan v2.31 step 2) — the SAME
//! documents `ce query --format json` and `ce rules --format json`
//! print: a question over the index's facts answered by the core's
//! query/1 with its derivations under `why`, and the project's rules
//! file judged. Rendering is the webview's; the answers, the proofs
//! and the errors are the core's, and a core without the family
//! shows the document's named degraded posture, never a verdict of
//! the screen's own. Housed beside commands.rs at its size line.

use crate::commands::face_cmd;
use codeeraser::faces::{self, doc, Plumb};
use codeeraser::similar::query::Ask;
use std::path::Path;

// `body` the question, `why` every answer's derivation, `file` the
// rules file it builds on (root-relative; absent = the project's).
face_cmd!(query_report, "query", (body: String, why: bool, file: Option<String>) =>
    |r, c, body: String, why, file: Option<String>| doc(faces::query(
        r, c, (&body, why), file.as_deref().map(Path::new), Plumb::default())));

// The text a question builds on (`ce query --prelude`): the built-in
// prelude and the program the core renders from `[rules.pack]`.
face_cmd!(query_prelude, "query_prelude", faces::pack_text);

// The clone-merge suggestions (plan v2.31 step 7) — the SAME document
// `ce merge --format json` prints, rendered by the reports hub's own
// merge card (hub_merge.js).
face_cmd!(merge_report, "merge", |r, c| doc(faces::merge(
    r,
    c,
    Plumb::default()
)));

// `file` the rules file to judge (root-relative; absent = `[rules]
// file`, else ce.rules), `why` every query answer's derivation too.
face_cmd!(rules_report, "rules", (file: Option<String>, why: bool) =>
    |r, c, file: Option<String>, why| doc(faces::rules(
        r, c, file.as_deref().map(Path::new), why, Plumb::default())));

// The architecture document (plan v2.31 step 9) — the SAME document
// `ce arch --format json` prints, rendered in the reports hub by
// hub_arch.js; `impact` are the root-relative paths whose dependents
// the core traces. Advisory like its CLI face.
face_cmd!(arch_report, "arch", (impact: Vec<String>) =>
    |r, c, i: Vec<String>| doc(faces::arch(r, c, &i, Plumb::default())));

// The same-role advisor (plan v2.29 step 6) — the SAME document
// `ce similar` prints: exactly one of `at` (file:line), `text` or
// `unit` (`name/arity`), the associative view under `widen`.
// Advisory: the screen renders the order and role bits the core
// answered, and a core without the family shows the document's named
// degraded posture, never a verdict of the webview's own.
face_cmd!(similar_report, "similar",
    (at: Option<String>, text: Option<String>, unit: Option<String>, widen: bool) =>
    |r, c, at: Option<String>, text: Option<String>, unit: Option<String>, widen| doc(
        faces::similar(r, c, &Ask::from_parts(at.as_deref(), text.as_deref(), unit.as_deref())?,
            widen, Plumb::default())));
