-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce clone` documents (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the faces
-- that assembled them on the measuring side: the T3 report
-- (cli/src/dedup/t3/mod.rs `Judged::report` through the shared
-- envelope, cli/src/report.rs) — the pairs the core's verdict bit
-- names clones, each with its raw tree edit distance and both sizes,
-- and the sixteen counters — and the unit universe `--units` lists
-- (cli/src/faces.rs `clone_units`). The measuring side sends every
-- judged pair with the clone/1 reply's scores and verdict bit (the
-- cache's replays among them), its own counters, and for `--units` one
-- row per unit; a unit is named by reference.
module CE.Clone.Document (doc, unitsDoc) where

import CE.Document.Contract
import CE.Document.Envelope
import Data.Aeson (Value, object, (.=))
import Data.Foldable (asum)

doc :: DocFamily
doc = docFamily "clone" (enSchema report) statement verdicts (enveloped report) []

-- | The T3 report: a pair row's metrics are [ted, n1, n2]; the
-- counters are the measuring side's, as the report names them.
report :: Envelope
report = envelopeOf (schemaId <> " clones unit units units | ted n1 n2 | over_cap_units forest_units survivors s5_windowed s5_pruned_label s5_already s5_new pairs_dropped_over_cap pairs_dropped_forest sent requests prefiltered judged cached")

statement :: String
statement = "range units\n" <> envelopeStatement report <> "ref unit units\n"

-- | The `--units` listing: one row per cached unit [u, file, nth,
-- nodes], in the index's order; the key is the unit's reference.
unitsDoc :: DocFamily
unitsDoc = docFamily "clone-units" unitsSchema unitsStatement unitsChecked unitsAssemble []

unitsSchema :: String
unitsSchema = "ce.clone-units/0.1.0"

unitsStatement :: String
unitsStatement = "range paths\nrange units\nrows units 4 judged units paths - -\nref path paths\nref key units\n"

unitsChecked :: DocReq -> Maybe String
unitsChecked req = asum [dense req "units" (range req "units")]

unitsAssemble :: DocReq -> Value
unitsAssemble req = object ["schema" .= unitsSchema, "units" .= map unit (rows req "units")]
 where
  unit r = case r of
    [u, f, nth, nodes] -> object ["path" .= ref "path" [f], "key" .= ref "key" [u], "nth" .= nth, "nodes" .= nodes]
    _ -> object []

-- | The document's schema id (the facts registry reads it here).
schemaId :: String
schemaId = "ce.clone-report/0.3.0"
