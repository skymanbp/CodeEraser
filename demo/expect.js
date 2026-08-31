"use strict";
// What this run MUST observe, in one place.
//
// Every channel the demo reads is fail-open by design: `ce probe --hook`
// and `ce audit --hook` never fail outward (a missing core, an unbuilt
// index, a broken ce.toml and a degraded diff all arrive as silence),
// `git` answers through an exit code a caller can drop, and
// `ce erase --apply` exits 0 for any row count, zero included. So a run
// whose tools quietly degraded renders a table shaped exactly like a
// measured one — and `bless` then freezes it, because the byte gate
// compares each file against its own generator and cannot see a
// generator that lied.
//
// Seeing that needs an expectation the run does not derive from its own
// output. Every constant below is a fact of the committed artefacts in
// demo/out: when the seed or the steps change, these change with them,
// and the diff says so.

const fs = require("fs");
const path = require("path");

/** The steps whose write the guard must refuse (out/with-codeeraser.txt). */
const DENIED = [1, 7];
/** What the first Stop audit's reason must name, since the scripted
 *  repair answers that reason and no other. */
const BLOCKED_ON = ["2 duplicate block", "invoicer/report.py"];
/** What `ce erase --apply` must report removing — the table's erased row
 *  states the twin by name, so the run has to check the name. */
const ERASED = ["erase applied: 1 row", "verbatim_doc docdup: twin of docs/DISCOUNTS.md"];

function must(ok, what) {
  if (!ok) throw new Error(`demo expectation: ${what}`);
}

/** The guard's answer to one write. Silence and `deny` are the only two
 *  outcomes this demo narrates; `ask` and an allow-carrying-a-warning
 *  would both be told as "landed (the guard had nothing to say)". */
function guard(step, verdict) {
  const denied = DENIED.includes(step.id);
  if (verdict) {
    must(
      verdict.decision === "deny",
      `step ${step.id} (${step.file}): the guard answered "${verdict.decision}", which this demo does not narrate`
    );
    must(verdict.reason.length > 0, `step ${step.id}: the refusal carries no reason`);
  }
  must(
    !!verdict === denied,
    denied
      ? `step ${step.id} (${step.file}): the guard let a write through that this demo shows refused`
      : `step ${step.id} (${step.file}): the guard refused a write that this demo shows landing`
  );
}

/** A gate's exit code. `ce` exits 1 for a failing gate and 2 for an error
 *  (main_cmds.rs fail()); the table renders every nonzero as FAIL, so a
 *  crashed gate would read as a finding. */
function gate(args, r) {
  must(
    r.rc === 0 || r.rc === 1,
    `ce ${args.join(" ")} exited ${r.rc} — a crash, not a verdict: ${r.out}`
  );
}

/** Both languages must reach the same verdict: one-sided silence would
 *  put contradictory stories about one run into the two READMEs. */
function agree(stop, when) {
  must(
    (stop.en === null) === (stop.zh === null),
    `the ${when} audit blocks in one language and not the other`
  );
}

/** The audit that must refuse the turn, on the grounds the repair answers. */
function firstAudit(stop) {
  agree(stop, "first");
  must(stop.en !== null, "the Stop audit let the turn end with the duplication still in the tree");
  for (const name of BLOCKED_ON) {
    must(
      stop.en.includes(name),
      `the audit blocked on something the scripted repair does not answer (no "${name}"): ${stop.en}`
    );
  }
}

/** The audit's own record of the two calls just made: a satisfied verdict
 *  and a failure to measure are both an empty stdout, and only the feed
 *  separates them (audit/observe.rs writes `degraded` and `skipped`). */
function measured(dir, n) {
  const feed = path.join(dir, ".ce", "observe.ndjson");
  must(fs.existsSync(feed), "the audit left no observe feed to check its silence against");
  const stops = fs
    .readFileSync(feed, "utf8")
    .trim()
    .split("\n")
    .map((l) => JSON.parse(l))
    .filter((e) => e.event === "stop_audit");
  must(stops.length >= n, `the feed holds ${stops.length} stop_audit lines, fewer than the ${n} just asked`);
  for (const e of stops.slice(-n)) {
    must(e.degraded === false, "the audit went silent because it could not measure, not because it was satisfied");
    must(e.skipped === undefined, `the audit skipped its measurement (${e.skipped})`);
  }
}

/** The audit that must go silent — and must have measured that silence. */
function repairedAudit(stop, dir) {
  agree(stop, "post-repair");
  must(stop.en === null, `the repair did not silence the audit: ${stop.en}`);
  measured(dir, 2); // the en and zh calls just made
}

/** What the eraser reports removing must be what the table says it
 *  removed. Its exit code alone cannot say so: an empty plan, and one
 *  whose row is some other file, both apply cleanly and exit 0. */
function erased(r) {
  must(r.rc === 0, `ce erase --apply exited ${r.rc}: ${r.out}`);
  for (const name of ERASED) must(r.out.includes(name), `the erase did not report "${name}": ${r.out}`);
}

/** The refusal count the transcript's closing note and the scoreboard's
 *  first row both rest on. */
function refusals(n) {
  must(n === DENIED.length, `${n} writes were refused, not the ${DENIED.length} this demo narrates`);
}

module.exports = { must, guard, gate, firstAudit, repairedAudit, erased, refusals };
