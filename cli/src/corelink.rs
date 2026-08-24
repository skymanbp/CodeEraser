//! Persistent NDJSON link to ce-core (ADR-003 wire format,
//! contracts/VERSIONING.md). `Link` holds the spawned core across
//! requests — strict lockstep, exactly one request outstanding; the
//! one-shot `run` (hello + EOF) remains for `ce doctor`.

mod pipe;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::process::{Child, Stdio};

/// Protocol version offered by this client (single source together
/// with core/app/CE/Protocol.hs::proto — contracts/VERSIONING.md §1).
/// 2.33.0 = the join-lattice minor (H4, 2026-08-24): the verdict
/// table gains a SEVERITY column (delete 3 > merge 2 > hotspot 1,
/// data the battery permutes), candidate rows widen to six columns
/// with the leg-agreement CONFIDENCE (how many present legs
/// corroborate — the attribution table CE.Verdict.Join.legBits),
/// and the reply ships the (code, severity) face once as
/// joinSeverity. ce join judges its pairs over the SAME verdict/1
/// road ce check gates with — one judgment, two faces; its exit
/// stays report-only.
/// 2.32.0 = the deadcode-confidence minor (H3, 2026-08-24): the
/// graph request may ship the per-language site ledger
/// ("unres": [[lang, unresolvedSites, totalSites]]) and each dead
/// row then grows a confidence column judged by the core — 0
/// unvouched / 1 vacuous / 2 vouched (CE.Graph.Cost.confidence,
/// the erase trust boundary executed by the ledger's owner). The
/// erase family gains class 3 (dead_file, confidence road: fact 1
/// is the core's confidence, refused only at 0); class 0 is
/// superseded, judged unchanged for the grace window — its
/// unknown-class specimen in the goldens legally re-answers as a
/// bounds refusal. Legacy graph requests keep two-column dead rows.
/// 2.31.0 = the trend/2 minor (H2, 2026-08-24): the slope
/// estimator becomes Theil-Sen — the median of pairwise slopes,
/// exact Rational; one wild point drags a least-squares mean
/// anywhere and cannot move a median past its neighbors (a RECORDED
/// behavior change: the same window may answer differently, and the
/// capability name says so — trend/2). The reply grows two shape
/// facts named by request index (cliff = steepest single-step fall,
/// declineRun = longest strictly-falling run) and counts.judged:
/// the judgment window caps at tsWindow = 512 recent points, the
/// O(n²) pair set measured ~150 ms end to end.
/// 2.30.0 = the fn-naming facts minor (ADR-008 batch-7 slice 14,
/// 2026-08-24): scan.request grows the aligned `naming` facts table
/// — [lang, style, upper, under, test], one row per code-6 row,
/// whose value must then be 0 (the verdict never crosses) — and the
/// convention judgment moves to CE.Scan.Cost.conforms. The godoc
/// underscore exemption is gated on Go's own lang code with go
/// vet's prefix boundary, killing the two defects the Rust
/// predicate carried (Testing_helper exempted; the exemption
/// leaking to every mixedCaps language). Legacy road (no naming
/// key) byte-identical; naming.rs stays as the pinned mirror.
/// 2.29.0 = the H1 triple minor (ADR-008 follow-ons, 2026-08-24):
/// ① the verdict knobs echo gains judgedMask — the judged-language
/// set as a Lang-code bitmask, declared by this client and pinned
/// per run (batch-7 slice 2's promised knob; the predicate stays
/// Rust's). ② the unused Markdown reference definition resolves and
/// travels as EDGE_REFDEF_UNUSED, the SECOND liveness-inert kind
/// the core drops (slice 16 executed; an unresolvable unused def is
/// an ordinary miss now, the borrowed External category retired;
/// GRAPH_REV 8). ③ the pre-judged staleDocs arm retired — its
/// 2.23.0 one-minor grace long expired; a legacy key falls to the
/// §1 unknown-field rule and axis 5 judges from raw tables alone.
/// 2.28.0 = the entry-roles minor (ADR-008 batch-7 slice 3 main
/// body): graph.request node rows accept an additive 4th column of
/// role FACTS (0 named main, 1 executable dir, 2 test convention,
/// 3 entry glob, 4 doc entry, 5 ce:allow claim, 6 declared build
/// target); the role→entry-bit table is CE.Graph.Cost.roleBits and
/// the legacy flags column yields wherever the roles ride (rows one
/// arity — mixed refuses). Role 6 closes the slice-3 defect: a
/// declared [[bin]] path / cabal main-is target is a root, where
/// before only name conventions were.
/// 2.27.0 = the code-only cycle-axis minor (M9 batch 9 P8, plan v2.12):
/// verdict.request gains additive docFiles, the ascending Markdown-file
/// indices; absent/empty preserves the old charge and Haskell owns the rule.
/// 2.26.0 = the one-density-law minor (M9 batch 9 P9): structure
/// score + axes rows move to the verdict density fold
/// (chargeAt shared from CE.Verdict.Score); axis rows carry
/// charges in permille now, findings unchanged.
/// 2.25.0 = the exemption-authority minor (M9 batch 7 slice 9):
/// docdup echo +licHeadLines (CE.Docdup.Cost = 5); exemption
/// execution stays pre-wire, string tables unpinned by decision.
/// 2.24.0 = the session-audit minor (M9 batch 7, slice 7): the
/// tenth family audit/1 -- [aTouched, bTouched] bit rows per clone
/// block; conviction (either side) and zero tolerance are the
/// core's (CE.Audit.Cost), the Stop/precommit legs relaying fail.
/// 2.23.0 = the raw-staleness minor (M9 batch 7, slice 11):
/// structure.request gains staleDocRows [dirId, docTs] + the
/// changed-target staleEdgeRows [docIdx, targetTs]; the S5 stale
/// predicate is the core's (deriveStale), the pre-judged staleDocs
/// rows legal one more minor and yielding to the raw tables.
/// 2.22.0 = the defect-sweep minor (M9 batch 7 close): the docdup
/// echo gains docLineCap (mirror-pinned; the SKELETON_PREFIXES
/// string table stays unpinned — numeric echo grammar, DOCDUP_REV
/// guards it) alongside the Rust-side defect sweep (bit-6 producer,
/// exemption tallies surfaced, real coverage bit, guard notice,
/// trend named refusal).
/// 2.21.0 = the pinned-floors minor (M9 batch 7, slices 5/10):
/// newBaseline gains zoneTiers [warn, ask] (the guard tier map,
/// core-authored, riding the committed baseline to the daemon-free
/// hook); the clone echo gains minUnitNodes and the docdup echo
/// minDocTokens — admission floors executed Rust-side pre-wire,
/// now core-owned and mirror-pinned per judged run.
/// 2.20.0 = the full-evidence minor (M9 batch 7, slices 12/13/15):
/// asset-kind edge rows travel and the core drops them from
/// liveness (CE.Graph.Cost.assetKind); the cochange table ships
/// whole at the configured floor (no client rank cut); clone-pair
/// language identity is the grammar, not the extension string.
/// Shapes unchanged — sending asset rows requires a core that
/// ignores them.
/// 2.19.0 = the diversity-floor minor (M9 batch 7, slice 1):
/// verdict.request gains additive dedupDistinct rows + the optional
/// dedupMinDistinct override; the core re-derives the admitted count
/// with CE.Dedup.Cost.minDistinct and judges the budget from ITS
/// number; the reply's dedupBlocks proves the local filter equal and
/// the knobs echo grows minDistinct (16 keys).
/// 2.18.0 = the RG9-repatriation minor (M9 batch 7, slice 4):
/// graph.result splits its verdicts core-side — `dead` carries only
/// file-granularity rows (the failing table, erase's licence
/// source), the additive `reported` table carries package/section
/// verdicts, and the additive `fail` bit names the zero-tolerance
/// gate (degraded fails by itself, the P1 stance). The client keeps
/// the split as a boundary contract: an aggregate in the failing
/// table refuses as wire skew, never licenses a directory erase.
/// 2.17.0 = the density-scoring minor (M9 batch 6): verdict/1 axes
/// rows become bounded per-axis charges — floor(scale·v/(v+n)) over
/// each axis's violation mass and opportunity count — and the score
/// their weighted mean under the violCost dial (neutral at 10); the
/// zone curve turns C¹ linear past H. Row shapes unchanged
/// (values-only migration, the 2.14.0 precedent): two real repos
/// field-measured at 0/1000 under the old mass-sum clamp.
/// 2.16.0 = the erase minor (M9 batch 3, plan v2.8 ②): the erase/1
/// family — the deterministic eraser's safety predicate. Dense
/// [class, w, x, y, z] fact rows in, [eraseable, reason] verdicts
/// out in request order; knobless by design (safety is not tunable),
/// paths never on the wire, degraded = fail with an empty table.
/// 2.15.0 = the ROI-pricing minor (plan v2.7 ②, v0.7): the split
/// advisory's two priced legs — structure.request gains the optional
/// `seamClones` block spans and `seamChurn` co-change pairs (legal
/// only beside seamFiles; absent = that leg unpriced), knobs 17/18
/// (roiCloneMilli / roiChurnMilli); the reply shape is unchanged —
/// both legs fold into costMilli.
/// 2.14.0 = the size-advisory minor (plan v2.6, v0.6, one additive
/// minor for both faces): verdict/1 grades axis 0 on the soft-zone
/// curve (declared score migration), gains judgedLoc + ceilings
/// codes 2..4 + newBaseline.softLine; structure/1 gains the
/// split-ROI advisory (seam tables in, splitCandidates/sizeExempt
/// out exactly when sent, knobs 12..16).
/// 2.12.0 = the staleness-axis minor (M6 S3c): structure.request
/// gains the optional `staleDocs` join rows (absent = axis 5
/// unjudged) and knob 11 — the seven-axis face closes.
/// 2.13.0 = the trend minor (M7.5b): the trend/1 family — the score
/// trajectory's [ts, score, scale] rows in, the exact-Rational
/// least-squares slope verdict out with the two-knob echo; below
/// minPoints answers null, fail trips only under a declared floor.
/// 2.11.0 = the redundancy-axis minor (M6 S3b): structure.request
/// gains the optional `redundancy` rollup rows (absent = axis 6
/// unjudged) and knobs 9/10.
/// 2.10.0 = the declared-layout minor (M6 S3a): structure.request
/// gains the additive `declared` [dirId, weight] rows and the reply
/// — only when a layout is declared — the `divergence` χ² row plus
/// the named `deviations` rows.
/// 2.9.0 = the structure minor (M6 S2): the structure/1 family —
/// tree-scale facts in, judged axes / entropy / findings out.
/// 2.8.0 = the review-repair minor (ADR-008 反审批): verdict replies
/// gain the effective `weights` table and the ratchet `failed` name
/// list; floor validates against the effective scale; self/duplicate
/// pairs refuse; caps cover the knob tables.
/// 2.7.0 = the scan-grading minor (ADR-008 P3): the scan/1 family —
/// measurement rows in, graded levels and the fail bit out, the
/// grade table echoed whole. 2.6.0 = the ratchet-unification minor
/// (ADR-008 P2): verdict.request gains the additive `dedup`
/// [blocks, budget] pair — the second ratchet's comparison is the
/// core's. 2.5.0 = the verdict-repatriation minor (ADR-008 P1):
/// clone/docdup replies carry per-row `verdicts` bits, the docdup
/// echo gains `verbatimFloor`, the degraded verdict reply fails by
/// itself. 2.4.0 = verdict.request `thresholds` + `tolerance` knob
/// rows and the FULL effective-knob echo (ADR-008 P4, additive).
/// 2.3.0 = `ceilings` rows + the two-knob echo (ADR-008 first
/// step). 2.2.0 = clone/1 + docdup/1 + verdict/1 in one additive
/// minor (M5-3a). 2.1.0 = graph/1 (M5-2a). 2.0.0 was the M5-1c-iii
/// anchor-width request shape (a breaking change, major per §2);
/// 1.0.0 was the M4 content finalization freeze.
pub const PROTO: &str = "2.33.0";

#[derive(Serialize)]
struct Hello<'a> {
    proto: &'a str,
    r#type: &'a str,
    client: &'a str,
    version: &'a str,
}

#[derive(Deserialize)]
pub struct HelloReply {
    pub proto: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub server: String,
    pub version: String,
    pub accept: bool,
    #[serde(default)]
    pub reason: Option<String>,
    /// Informational discovery only — SemVer stays the sole authority
    /// for accept/reject (§1). Absent capability = run L1, degraded.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// A live core process past its accepted hello. Replies arrive via
/// pipe::reader's channel so every wait carries a deadline — an
/// unbounded read_line let a wedged core hold the daemon and every
/// hook behind it forever.
pub struct Link {
    child: Child,
    replies: std::sync::mpsc::Receiver<std::io::Result<String>>,
    deadline: std::time::Duration,
    caps: Vec<String>,
    next_id: u64,
}

impl Link {
    /// Spawn `core` and perform the handshake; the link stays open.
    pub fn open(core: &str) -> Result<(Link, HelloReply), String> {
        let mut child = spawn(core)?;
        let replies = pipe::reader(child.stdout.take().ok_or("no stdout pipe")?);
        let mut link = Link {
            child,
            replies,
            deadline: pipe::deadline(),
            caps: Vec::new(),
            next_id: 0,
        };
        let hello = Hello {
            proto: PROTO,
            r#type: "hello",
            client: "ce",
            version: env!("CARGO_PKG_VERSION"),
        };
        let line = serde_json::to_string(&hello).map_err(|e| e.to_string())?;
        link.send(&line)?;
        let parsed = serde_json::from_str(&link.read_line()?)
            .map_err(|e| format!("bad hello reply: {e}"))?;
        let reply = validate(parsed)?;
        link.caps = reply.capabilities.clone();
        Ok((link, reply))
    }

    pub fn has(&self, capability: &str) -> bool {
        self.caps.iter().any(|c| c == capability)
    }

    /// One `{kind}.request` line out, one `{kind}.result` line in.
    /// Stamps proto/type/id; a reply that does not echo the id or
    /// carry the expected type is a desync — the caller falls back to
    /// L1, visibly (A9f).
    pub fn request(&mut self, kind: &str, mut body: Value) -> Result<Value, String> {
        self.next_id += 1;
        let obj = body
            .as_object_mut()
            .ok_or("request body must be an object")?;
        obj.insert("proto".into(), PROTO.into());
        obj.insert("type".into(), format!("{kind}.request").into());
        obj.insert("id".into(), self.next_id.into());
        let line = serde_json::to_string(&body).map_err(|e| e.to_string())?;
        self.send(&line)?;
        let reply: Value =
            serde_json::from_str(&self.read_line()?).map_err(|e| format!("bad reply: {e}"))?;
        let expected = format!("{kind}.result");
        // an error reply that echoes our id is a REFUSAL, not a
        // desync: surface the core's named reason (review C4 — the
        // knob roads put ce.toml values behind these messages, and
        // "desync" hid every one of them)
        if reply["type"] == "error" && reply["id"] == self.next_id {
            return Err(format!(
                "core refused {kind}.request: {}: {}",
                reply["code"].as_str().unwrap_or("?"),
                reply["message"].as_str().unwrap_or("?")
            ));
        }
        if reply["type"] != expected.as_str() || reply["id"] != self.next_id {
            return Err(format!("desync: expected {expected} id {}", self.next_id));
        }
        Ok(reply)
    }

    fn send(&mut self, line: &str) -> Result<(), String> {
        let stdin = self.child.stdin.as_mut().ok_or("no stdin pipe")?;
        writeln!(stdin, "{line}").map_err(|e| format!("write: {e}"))?;
        stdin.flush().map_err(|e| format!("flush: {e}"))
    }

    fn read_line(&mut self) -> Result<String, String> {
        pipe::next_line(&self.replies, self.deadline, &mut self.child)
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        drop(self.child.stdin.take()); // EOF: the polite exit
        // then make exit unconditional — a bare wait() on a wedged
        // core blocked Drop forever, and the core keeps no state a
        // kill could corrupt (pure stdin/stdout judge)
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// One-shot hello for `ce doctor` (M0 behaviour, kept by CI).
pub fn run(core: &str) -> Result<HelloReply, String> {
    Link::open(core).map(|(_link, reply)| reply)
}

/// The effective core binary for a `--core` flag value: an explicit
/// path is used verbatim; the untouched default routes through the
/// daemon's resolver chain — CE_CORE_BIN, a ce-core SIBLING of this
/// executable (the installed layout drops both binaries side by
/// side), then PATH. One authority with daemon/MCP (core_bin), and
/// applied at the ONE spawn throat below, so every judgment family
/// resolves identically with no per-flag plumbing.
pub fn resolve_core(core: &str) -> String {
    if core != "ce-core" {
        return core.to_string();
    }
    crate::daemon::judge::core_bin().unwrap_or_else(|| core.to_string())
}

fn spawn(core: &str) -> Result<Child, String> {
    let effective = resolve_core(core);
    crate::proc::command(&effective)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start `{effective}`: {e}"))
}

fn validate(reply: HelloReply) -> Result<HelloReply, String> {
    if reply.kind != "hello" || reply.server != "ce-core" {
        return Err(format!(
            "unexpected reply type/server: {}/{}",
            reply.kind, reply.server
        ));
    }
    if major(&reply.proto)? != major(PROTO)? {
        return Err(format!(
            "proto mismatch: core {} vs ce {PROTO}",
            reply.proto
        ));
    }
    match reply.accept {
        true => Ok(reply),
        false => Err(reply
            .reason
            .unwrap_or_else(|| "core rejected handshake".into())),
    }
}

fn major(v: &str) -> Result<u64, String> {
    v.split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("bad semver `{v}`"))
}

#[cfg(test)]
mod tests {
    /// An explicit --core path travels verbatim; the untouched
    /// default consults the ONE resolver the daemon and MCP already
    /// use — equality against core_bin pins the single-authority
    /// property without poking the process environment (the e2e
    /// suite owns CE_CORE_BIN; mutating it here would race them).
    #[test]
    fn explicit_core_wins_and_default_routes_through_the_one_resolver() {
        assert_eq!(super::resolve_core("x/custom/core"), "x/custom/core");
        assert_eq!(
            super::resolve_core("ce-core"),
            crate::daemon::judge::core_bin().expect("resolver always answers")
        );
    }
}
