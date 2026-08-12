-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | fourclass message shapes (contracts/VERSIONING.md; golden:
-- contracts/fixtures/fourclass/). Hashes are u64 and must decode
-- exactly — aeson routes them through Scientific's Integer
-- coefficient, never a Double; the golden fixture pins the boundary
-- value 18446744073709551615. Nothing text-shaped crosses the wire:
-- pair indices, line numbers, hashes (ADR-002 A6).
module CE.FourClass.Wire
  ( Request (..)
  , Pair (..)
  , Result (..)
  , Block (..)
  , encodeResult
  ) where

import Data.Aeson hiding (Result)
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Word (Word64)

-- | One request: the L1 leftovers (significant lines classified
-- novel/deleted) per changed file pair. `id` is echoed opaquely.
data Request = Request
  { reqId :: Value
  , reqPairs :: [Pair]
  }

-- | `i` is an opaque pair index — the file-identity key (cross
-- matching requires differing `i`). `rem`/`add` are ascending
-- (1-based line, trimmed-content hash, alnum width) lists grouped
-- into RUNS by the aligner: run structure is alignment data, so Rust
-- produces it (blank/punctuation changed lines bridge a run;
-- unchanged gaps and within-moved lines break it) and judgment
-- consumes it as given. The width (proto 2.0.0) is a LINE FACT the
-- aligner measures — the judgment it feeds (Cost.anchorFloor) lives
-- entirely here.
data Pair = Pair
  { pIdx :: Int
  , pRem :: [[(Int, Word64, Int)]]
  , pAdd :: [[(Int, Word64, Int)]]
  , pDup :: [Word64]
  -- ^ key hashes of units newly DUPLICATED on the after side --
  -- symbol knowledge stays in Rust (ADR-002), only hashes cross
  }

instance FromJSON Pair where
  parseJSON = withObject "Pair" $ \o ->
    Pair <$> o .: "i" <*> o .: "rem" <*> o .: "add" <*> o .:? "dup" .!= []

instance FromJSON Request where
  parseJSON = withObject "Request" $ \o ->
    Request <$> o .: "id" <*> o .: "pairs"

-- | An accepted cross-pair correspondence: positionally matching
-- line lists, length >= the derived floor. Lines admitted by
-- extension or source attribution appear in `moved` but not here —
-- they are a relocation's tail, not its evidence.
data Block = Block
  { bFromPair :: Int
  , bFromLines :: [Int]
  , bToPair :: Int
  , bToLines :: [Int]
  }

-- | The reply: a monotone reclassification delta per pair (out:
-- removed_deleted -> removed_moved; in: added_novel -> added_moved)
-- plus the block evidence that justifies it.
data Result = Result
  { resId :: Value
  , resMoved :: [(Int, [Int], [Int])]
  , resBlocks :: [Block]
  , resSuspicions :: [(Int, String)]
  , resDegraded :: Maybe String
  }

encodeResult :: String -> Result -> B8.ByteString
encodeResult proto r =
  BL.toStrict . encode . object $
    [ "proto" .= proto
    , "type" .= ("fourclass.result" :: String)
    , "id" .= resId r
    , "moved" .= resMoved r
    , "blocks" .= [(bFromPair b, bFromLines b, bToPair b, bToLines b) | b <- resBlocks r]
    , "suspicions" .= resSuspicions r
    , "degraded" .= maybe False (const True) (resDegraded r)
    ]
      <> maybe [] (\why -> ["reason" .= why]) (resDegraded r)
