-- | The scan request's boundary contract — the machine check that
-- every row of every table is well shaped, every code and language
-- in range, every ladder coherent. Split from CE.Scan at the
-- 300-line dogfood wall when the judged-language mask arrived
-- (7.2.0; the CE.Graph.Contract precedent): checking what arrived
-- and judging it are two jobs, and only the first one is allowed to
-- know the request's spelling. Every row shape reads through
-- CE.Wire's rowCheck skeleton — the clone gate named the hand-rolled
-- copy (its rowShape rhymed with trend/2's) the moment it moved.
--
-- Every message here is golden-pinned text. A refusal is named by
-- table, row index and reason — or, for the whole-request mask, by
-- key and reason — so a producer learns which row it got wrong,
-- never just that something was wrong.
module CE.Scan.Contract (violation) where

import CE.Scan.Cycles (callBattery)
import CE.Scan.Fence (fenceOffence)
import CE.Wire (RowsReq (..), Rulepack (..), judgedLang, maskOffence, rowCheck, tableOffence)
import Control.Applicative ((<|>))
import Data.Foldable (asum)
import Data.List (find)
-- the rulepack fence is ONE predicate for every family that carries
-- a class id (the verdict family minted it at 3.1.0; 6.4.0 spelled
-- the inclusive bound once, O38)
import CE.Verdict.Cost (classIdPastFence)

-- | First boundary-contract offender in request order (Clone.hs
-- posture: the message names the violator deterministically); the
-- ascending pass compares CODES alone (warn values legitimately
-- vary), through CE.Wire's shared checker. The judged-language mask
-- (7.2.0) is a whole-request fact the naming rows are read against,
-- so its own shape is named before any row of any table is.
violation :: RowsReq -> Maybe String
violation req =
  asum
    [ maskOffence (maskOf req)
    , asum (zipWith rowShape [0 :: Int ..] (rowsOf req))
    , tableOffence "grade" (take 1) gradeShape (gradesOf req)
    , namingBattery req
    , callBattery (rowsOf req) (callsOf req)
    , classBattery req
    , fenceOffence (fenceOf req)
    ]

-- | The naming-facts table's own contract (2.30.0): aligned 1:1
-- with the code-6 rows in request order, each row the five shape
-- facts — and the verdict provably absent from the wire: a code-6
-- row must carry value 0 when facts ride (the staleDocs lesson,
-- inverted before shipping this time: one judgment, one road).
namingBattery :: RowsReq -> Maybe String
namingBattery req = case namingOf req of
  Nothing -> Nothing
  Just naming ->
    asum
      [ counts naming
      , asum (zipWith (namingShape (maskOf req)) [0 :: Int ..] naming)
      , asum (zipWith preJudged [0 :: Int ..] (rowsOf req))
      ]
 where
  counts naming
    | length naming /= fnRows =
        Just ("naming: " <> show (length naming) <> " rows for " <> show fnRows <> " fn-naming rows")
    | otherwise = Nothing
  fnRows = length [() | (6 : _) <- rowsOf req]
  preJudged i row = case row of
    [6, v] | v /= 0 -> Just ("row " <> show i <> ": pre-judged fn-naming value (naming facts ride)")
    _ -> Nothing

-- | The rulepack channel's own contract (3.2.0): a class column
-- aligned 1:1 with the rows, every class below the fence; override
-- rows [classId, code, warn, fail] from class 1 below the fence
-- (class 0 IS the global table, which `grades` already overrides),
-- a known code, a coherent ladder — the gradeShape reading, one
-- class dimension wider — and (classId, code) strictly ascending.
classBattery :: RowsReq -> Maybe String
classBattery req =
  (rowClassesOf rp >>= aligned)
    <|> tableOffence "gradeOverride" (take 2) overrideShape (overridesOf rp)
 where
  rp = rulepackOf req
  n = length (rowsOf req)
  aligned cs
    | length cs /= n = Just ("rowClasses: " <> show (length cs) <> " classes for " <> show n <> " rows")
    | otherwise = past <$> find (\(_, c) -> c < 0 || classIdPastFence c) (zip [0 :: Int ..] cs)
  past (i, _) = "rowClasses " <> show (i :: Int) <> ": class beyond the fence"

overrideShape :: Int -> [Integer] -> Maybe String
overrideShape = rowCheck "gradeOverride" "malformed row (need [class,code,warn,fail])" 4 checks
 where
  checks row = case row of
    (c : rest)
      | c < 1 -> Just "class 0 has no override channel"
      | classIdPastFence c -> Just "class beyond the fence"
      | otherwise -> ladderFault rest
    _ -> Nothing

-- | One naming-facts row; the language code must be in the judged
-- set the request declared (7.2.0), the legacy seven when it declared
-- none — the set is the producer's LANGS table, not a constant here.
namingShape :: Maybe Integer -> Int -> [Integer] -> Maybe String
namingShape mask = rowCheck "naming" "malformed row (need [lang,style,upper,under,test])" 5 checks
 where
  checks row = case row of
    [lang, style, upper, under, test]
      | not (judgedLang mask lang) -> Just "lang outside the judged set"
      | style < 0 || style > 2 -> Just "unknown style"
      | any (`notElem` [0, 1]) [upper, under, test] -> Just "non-boolean fact"
    _ -> Nothing

rowShape :: Int -> [Integer] -> Maybe String
rowShape = rowCheck "row" "malformed row (need [code,value])" 2 checks
 where
  checks row = case row of
    [code, v]
      | code < 0 || code > 6 -> Just "unknown metric code"
      | v < 0 -> Just "negative value"
      | v >= 18446744073709551616 -> Just "value outside u64"
    _ -> Nothing

-- | A grade override must stay a coherent ladder: a hard line BELOW
-- the warn line is refused, never silently reordered. fail == warn
-- is deliberately legal (review C19 ruling): it is the single-line
-- config — every breach grades 2 and the warn band is empty by the
-- user's own choice, which gradeWith honors on both sides of the
-- mirror.
gradeShape :: Int -> [Integer] -> Maybe String
gradeShape = rowCheck "grade" "malformed row (need [code,warn,fail])" 3 ladderFault

-- | One [code, warn, fail] ladder's first held fault, by name — the
-- grade table's rows and (3.2.0) the tail of every override row
-- read through the same predicate; the caller's rowCheck wears the
-- label.
ladderFault :: [Integer] -> Maybe String
ladderFault row = case row of
  [code, warn, failLine] ->
    snd
      <$> find
        fst
        [ (code < 0 || code > 6, "unknown metric code")
        , (warn < 0 || failLine < 0, "negative field")
        , (failLine /= 0 && failLine < warn, "fail line below warn")
        ]
  _ -> Nothing
