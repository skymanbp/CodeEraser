-- | The `ce docdup` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/docdup/judge/mod.rs `run`
-- and `Counts` through the shared envelope, cli/src/report.rs): the
-- segment pairs the core's verdict bit names duplicates, each with its
-- raw intersection and union and its verbatim run, and the thirteen
-- counters. The measuring side sends its live segments (file, span and
-- kind code — the kind's word is the definition package's
-- `docdup.kind_names`, CE.Lang.segmentKinds), every judged pair with
-- the docdup/1 reply's scores and verdict bit, and its counters; a
-- segment in the document is a reference, its kind spelled by the
-- package on both sides.
module CE.Docdup.Document (doc, segments) where

import CE.Document.Contract
import CE.Document.Envelope
import CE.Lang (segmentKinds)
import Data.Foldable (asum)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = docFamily "docdup" (enSchema report) statement checked (enveloped report) []

-- | The report: a pair row's metrics are [inter, union, verbatim run];
-- the counters are the measuring side's, as the report names them.
report :: Envelope
report = envelopeOf "ce.docdup-report/0.1.0 dups seg segments segs | inter union verbatim | over_cap_segments lsh_pairs seed_pairs hot_bands hot_shingles sent requests judged jaccard_dups exempt_license exempt_allow"

-- | A segment row is [s, file, start, end, kind], one per live
-- segment; `check` the console's `--check`.
statement :: String
statement =
  "range paths\nrange segs\nfact check judged\n"
    <> envelopeStatement report
    <> "rows segs 5 judged segs paths - - -\nref seg segs\nref path paths\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ dense req "segs" (range req "segs")
    , codes req "segs" 4 0 (toInteger (length segmentKinds) - 1)
    , verdicts req
    , bits req ["check"]
    ]

-- | Each segment's place for the console: its file, span and kind word.
segments :: DocReq -> M.Map Integer (Integer, Integer, Integer, String)
segments req = M.fromList [(i, (f, a, b, nameOf segmentKinds k)) | [i, f, a, b, k] <- rows req "segs"]
