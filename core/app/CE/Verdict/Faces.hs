-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The verdict reply's faces, split from CE.Verdict at the 300-line
-- core gate: what a reply SAYS — the named fail rows, the ratchet
-- object, the newBaseline document, the digest echo — as opposed to
-- how it was judged. Every consumer reads these by key name, so the
-- key set is the contract and lives in one place.
module CE.Verdict.Faces (digestKey, failConditions, newBaselineObj, ratchetObj) where

import CE.Verdict.Cost (zoneAskPermille, zoneWarnPermille)
import CE.Verdict.Ratchet (Ratcheted (..))
import Data.Aeson
import Data.Aeson.Types (Pair)

-- | The ADR-006 fail conjunction as NAMED rows (ADR-008 table form):
-- any held condition fails — over ceiling past tolerance, a new
-- discrete member, the --fail-under floor ("either alone fails"),
-- the dedup blocks-over-budget half (P2: `ce dedup --check` sends
-- the pair; the ce check road never does), the knob fence, and
-- (6.4.0, O40) a ratcheted row whose file is present yet unmeasured.
failConditions :: Ratcheted -> Bool -> Bool -> Bool -> [(String, Bool)]
failConditions r floorFail dedupOver digestDrift =
  [ ("ratchet_over", not (null (rOver r)))
  , ("discrete_added", not (null (rAdded r)))
  , ("floor", floorFail)
  , ("dedup_budget", dedupOver)
  , ("knobs_digest", digestDrift)
  , ("rows_dropped", not (null (rDropped r)))
  ]

-- | The ratchet face: the delta, the gate bit, and the NAMES of the
-- conditions that held (review C8, 2.8.0 additive) — consumers
-- attribute a failure by name instead of by construction-time
-- coincidence, which is the whole reason the rulepack fence (5.1.0)
-- could become a fourth way to fail without any consumer guessing.
-- Split from result at the E01 75-line function gate when it did.
ratchetObj :: Bool -> Ratcheted -> [(String, Bool)] -> Value
ratchetObj presentRode r conds =
  object $
    [ "added" .= rAdded r
    , "removed" .= rRemoved r
    , "over" .= rOver r
    , "toleranceDrawn" .= rDrawn r
    , "fail" .= any snd conds
    , "failed" .= [name | (name, True) <- conds]
    ]
      -- the dropped rows ride exactly when `present` rode (6.4.0):
      -- an empty list is an answer, an absent key is an older core
      <> ["dropped" .= rDropped r | presentRode]

-- | The newBaseline face. softLine (2.14.0, plan v2.6 §B): derived
-- from judgedLoc at establish, carried verbatim otherwise — the
-- re-anchor is CE_ACCEPT_BASELINE by construction, because only
-- establish reaches the derivation. zoneTiers (batch-7 slice 5,
-- 2.21.0, additive): the zone tier cut points ride the baseline to
-- the daemon-free hook — core-authored, locally read. Split from
-- result when the 2.33.0 severity face pushed it past the 75-line
-- hard line the repo dogfoods.
--
-- The digest (5.1.0, whole-config since 6.0.0) is echoed on EVERY
-- reply that carried one — this ordinary result path included, and
-- since 6.4.0 (O43) the degraded reply too — and a config equal to
-- the shipped default sends none, so the key stays absent (K11).
-- What makes RECORDING it a named act is not this function: a
-- different digest reaches disk only through `ce baseline` under
-- CE_ACCEPT_BASELINE=1 (wholesale) or CE_ACCEPT_FENCE=1 (the narrow
-- re-pin) — that gate lives in cli/src/main_score.rs, in the other
-- language, and baseline::write is a mechanism any in-process
-- caller may drive. The guarantee is "no CLI road persists a
-- drifted digest without an act", not a type-level fence.
newBaselineObj :: Ratcheted -> Maybe Integer -> Maybe Integer -> Value
newBaselineObj r newSoft digest =
  object $
    [ "continuous" .= rNewCont r
    , "discrete" .= rNewDisc r
    , "softLine" .= newSoft
    , "zoneTiers" .= [zoneWarnPermille, zoneAskPermille]
    ]
      <> digestKey digest

-- | The digest key, present exactly when the request carried one:
-- absent ⇔ none sent, on the judged and the degraded reply alike, so
-- the client asserts the echo with no special case (O32).
digestKey :: Maybe Integer -> [Pair]
digestKey digest = ["knobsDigest" .= d | Just d <- [digest]]
