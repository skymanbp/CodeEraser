-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | verdict.request shape and boundary contract (design §2.2), split
-- from CE.Verdict at the 300-line core gate. The tier table is the
-- node universe (dense: row i names node i); every fact table
-- indexes into it, strictly ascending on its IDENTITY PREFIX (a
-- duplicate identity with a different payload is exactly the drift
-- this refuses to let in), and the first offender is reported in a
-- deterministic order: universe first, then facts as the request
-- lists them (the Graph.hs convention).
module CE.Verdict.Wire
  ( VerdictReq (..)
  , violation
  ) where

import CE.Verdict.Ratchet (Baseline (..))
import CE.Verdict.Rows
  ( contRow
  , discEntry
  , docFilesOffence
  , floorOffence
  , judgedLocOffence
  , nodeRow
  , pairRow
  , posRow
  , presentOffence
  , selfLoopsOffence
  , simRow
  , tierRow
  )
import CE.Verdict.Table
  ( ascendingBy
  , ceilingsOffence
  , classKnobsOffence
  , dedupDistinctOffence
  , dedupOffence
  , table
  , thresholdsOffence
  , toleranceOffence
  , uniformArity
  , weightsOffence
  )
import Control.Applicative ((<|>))
import Data.Aeson
import Data.Foldable (asum)
import qualified Data.IntSet as IS

data VerdictReq = VerdictReq
  { reqId :: Value
  , reqSim :: [[Integer]]
  , reqPos :: [[Integer]]
  , reqTier :: [[Integer]]
  , reqChurn :: [[Integer]]
  , reqCochange :: [[Integer]]
  , reqCont :: [[Integer]]
  , reqDisc :: [Integer]
  , reqBaseline :: Value
  , reqWeights :: [[Integer]]
  , reqFloor :: Maybe Integer
  , -- ADR-008 first step (2.3.0, additive — absent parses as []):
    -- [axisCode, ceiling] rows overriding the size (0) and coc (1)
    -- axis ceilings; ce.toml is the source, this wire is the road,
    -- and the Cost.hs values become DEFAULTS instead of the second
    -- half of an uncheckable mirror (M5-close audit D2)
    reqCeilings :: [[Integer]]
  , -- ADR-008 P4 (2.4.0, additive): the remaining verdict-family
    -- knobs speak the same [code, value] grammar — thresholds codes
    -- 0..6 (deadIndegCeil / rewriteNum / rewriteDen / cochangeFloor
    -- / violCost / defaultWeight / scoreScale) and the ADR-006
    -- tolerance legs 0..2 (tolNum / tolDen / tolAbs). Absent = []
    -- = every knob at its Cost.hs DEFAULT.
    reqThresholds :: [[Integer]]
  , reqTolerance :: [[Integer]]
  , -- ADR-008 P2 (2.6.0, additive): the dedup budget pair
    -- [blocks, budget] — the second ratchet's verdict inputs, sent
    -- by `ce dedup --check` alone. Absent = the condition is not
    -- evaluated (the ce check road is untouched).
    reqDedup :: Maybe [Integer]
  , -- batch-7 slice 1 (2.19.0, additive): the PRE-filter per-block
    -- distinct counts, sent beside the pair by `ce dedup --check` —
    -- the core re-derives the admitted block count with its own
    -- diversity floor (CE.Dedup.Cost.minDistinct, override below)
    -- and judges the budget from THAT; the reply's dedupBlocks lets
    -- the client prove its filter equal. Values only, no text.
    reqDedupDistinct :: [Integer]
  , -- the effective floor when the CLI overrode --min-distinct;
    -- absent = the core default judges (the trend-knob pattern).
    reqDedupFloor :: Maybe Integer
  , -- plan v2.6 §B (2.14.0, additive): the JUDGED-language file-LOC
    -- multiset the soft line derives from at establish. Values
    -- only — no entities, no paths (§5.9.2); the size axis itself
    -- keeps judging the wider continuous table (the v2.5 scan-only
    -- arm stays size-gated). Absent = [] = no derivable S.
    reqJudgedLoc :: [Integer]
  , -- plan v2.12 (2.27.0, additive): ascending file-universe indices
    -- of documentation-language files; absent/empty preserves old cycle
    -- scoring semantics.
    reqDocFiles :: [Integer]
  , -- H1 slice 2 (2.29.0, additive): the judged-language set as a
    -- Lang-code bitmask — batch-7 dispositioned the PREDICATE to
    -- Rust (the boundary authority) and promised the SET as an
    -- echo-pinned knob; 0 = not declared (an old client or the
    -- dedup-only road). The echo makes the set core-visible and
    -- drift-detectable; no judgment consumes it yet.
    reqJudgedMask :: Integer
  , -- plan v2.13 ① (3.1.0, additive): the rulepack's knob rows
    -- [classId, code, value] — the ceilings codes 0/1/2 (sizeCeil /
    -- cocCeil / sizeHard) under a class; the continuous rows carry
    -- the class as their 4th column. Class names and globs never
    -- cross (§5.9.2). Absent = [] = every row judges on the global
    -- lines, byte-identical to the legacy road.
    reqClassKnobs :: [[Integer]]
  , -- plan v2.14 ② (5.1.0, widened at 6.0.0): the KNOB FINGERPRINT —
    -- a scalar over the whole parsed ce.toml, every table and not
    -- the class table alone (`[score] viol_cost = 0` moved every
    -- gate through the narrow version and was measured doing it).
    -- Names and globs still never cross (§5.9.2); a hash of them is
    -- not them. Absent = the shipped default. The baseline records
    -- the digest it was established under, and a mismatch is a NAMED
    -- fail, so "edit a knob and every ceiling quietly moves" becomes
    -- a hard stop instead of a possibility.
    reqKnobsDigest :: Maybe Integer
  , -- plan v2.14 K15 (6.1.0, additive): the EXPORT SURFACE, the same
    -- [node, visibility] table graph/1 has carried since 4.1.0, keyed
    -- to the tier universe instead of the node universe. It exists so
    -- RG10 can hold on THIS road: the lattice's publicGuard reads
    -- flag bit 0, no producer ever set it here, and the guard was
    -- inert in production while the graph face had the fact all
    -- along. The raw visibility word crosses, never a derived
    -- "exported" list — which bit means exported is judgment
    -- (Graph.Cost.exportVisBit) and stays in the core. Absent = [] =
    -- every flag word 0, byte-identical to the legacy road.
    reqSymbols :: [[Integer]]
  , -- plan v2.18 step #14 (6.4.0, O40): the PROVENANCE table —
    -- ascending file entities that exist under the scope but own no
    -- continuous row this run. The ratchet reads a baseline row of
    -- one as DROPPED (a named fail) where a vanished entity stays a
    -- removal; only the client can see the disk, so only it can say
    -- which. Absent = the legacy road; empty = every candidate was
    -- measured (the reply still carries `dropped`, so an old core
    -- is told apart from a clean one).
    reqPresent :: Maybe [Integer]
  , -- plan v2.18 step #14 (6.4.0, O59): the file indices carrying a
    -- self-arc, required exactly at cycleFloor 1 (thresholds code 7)
    -- and refused elsewhere — CE.Verdict.Rows.selfLoopsOffence.
    reqSelfLoops :: Maybe [Integer]
  }

instance FromJSON VerdictReq where
  parseJSON = withObject "VerdictReq" $ \o ->
    VerdictReq
      <$> o .: "id"
      <*> o .: "sim"
      <*> o .: "pos"
      <*> o .: "tier"
      <*> o .: "churn"
      <*> o .: "cochange"
      <*> o .: "continuous"
      <*> o .: "discrete"
      <*> o .: "baseline"
      <*> o .: "weights"
      <*> o .: "floor"
      <*> o .:? "ceilings" .!= []
      <*> o .:? "thresholds" .!= []
      <*> o .:? "tolerance" .!= []
      <*> o .:? "dedup"
      <*> o .:? "dedupDistinct" .!= []
      <*> o .:? "dedupMinDistinct"
      <*> o .:? "judgedLoc" .!= []
      <*> o .:? "docFiles" .!= []
      <*> o .:? "judgedMask" .!= 0
      <*> o .:? "classKnobs" .!= []
      <*> o .:? "knobsDigest"
      -- the export surface (6.1.0): absent is the legacy road, and
      -- an empty table says the same thing an absent one does — no
      -- file here declares an export, so no flag word carries bit 0
      <*> o .:? "symbols" .!= []
      <*> o .:? "present"
      <*> o .:? "cycleSelfLoops"

-- | First boundary-contract offender, if any. The row checkers are
-- top-level functions taking the universe size n (the M5-close warn
-- repayment: a 64-line where block was the E01 offender, and the
-- checkers never needed the closure — only n and the tier table).
-- The baseline arrives PRE-PARSED: CE.Verdict parses it exactly once
-- and both the row cap and this check consume that result (the
-- M5-close LOW "parseBaseline runs twice per request", repaid
-- together with the baseline cap escape).
violation :: Either String (Maybe Baseline) -> VerdictReq -> Maybe String
violation parsed req =
  asum
    [ asum (zipWith tierRow [0 :: Int ..] (reqTier req))
    , table "sim" (simRow n) 2 (reqSim req)
    , asum (zipWith (posRow unitTier n) [0 :: Int ..] (reqPos req))
        <|> ascendingBy "pos" 1 (reqPos req)
    , table "churn" (nodeRow n 3) 1 (reqChurn req)
    , table "cochange" (pairRow n 3) 2 (reqCochange req)
    , table "continuous" contRow 2 (reqCont req)
        <|> uniformArity "continuous" (reqCont req)
    , asum (zipWith discEntry [0 :: Int ..] (reqDisc req))
    , ascendingBy "discrete" 1 (map pure (reqDisc req))
    , either Just (const Nothing) parsed
    , weightsOffence (reqWeights req)
    , ceilingsOffence (reqCeilings req)
    , thresholdsOffence (reqThresholds req)
    , toleranceOffence (reqTolerance req)
    , -- the rulepack's knob rows (3.1.0): the ceilings grammar one
      -- class dimension wider, judged before the floor like its kin
      classKnobsOffence (reqClassKnobs req)
    , floorOffence (reqThresholds req) (reqFloor req)
    , dedupOffence (reqDedup req)
    , dedupDistinctOffence (reqDedup req) (reqDedupDistinct req) (reqDedupFloor req)
    , judgedLocOffence (reqJudgedLoc req)
    , docFilesOffence n (reqDocFiles req)
    , -- the export surface (6.1.0) is graph/1's symbols table under
      -- another universe, so it is checked the way its kin are: two
      -- non-negative fields, the node in range, and the WHOLE row
      -- ascending — a deduped set, because two exported declarations
      -- in one file are not two facts about that file
      table "symbols" (nodeRow n 2) 2 (reqSymbols req)
    , if reqJudgedMask req < 0 then Just "judgedMask: negative" else Nothing
    , -- the 6.4.0 tables: provenance entities (u64, ascending) and
      -- the self-loop set, whose presence is tied to the cycle floor
      maybe Nothing presentOffence (reqPresent req)
    , selfLoopsOffence n (reqThresholds req) (reqSelfLoops req)
    ]
 where
  n = toInteger (length (reqTier req))
  -- built lazily, consulted only after the tier element of the asum
  -- has proven density (row i names node i) — the review HIGH-2
  -- repayment: the old per-row list scan re-derived that index and
  -- cost F²/2 across a legal request
  unitTier =
    IS.fromList
      [i | (i, [_, code]) <- zip [0 :: Int ..] (reqTier req), code /= 0]
