{-# LANGUAGE OverloadedStrings #-}

-- | What the TS and Rust rungs need of the measuring side (plan v2.33
-- W2-text stage E, the Rust ops since stage F), asked as facts instead
-- of read: whether a configuration path is a file and its text
-- (`nearest_up`'s probe, then `read_jsonc`), whether a compiled
-- JavaScript twin is a file (`esm_rewrite`), whether `node_modules/<name>`
-- under a directory is a directory (`bare_rung`); a Cargo.toml's decoded
-- document (`cargo::package`: TOML decoding is a library read on that
-- side, as the compile databases' JSON is), and what tree-sitter reads of
-- a Rust file — its answers at one row (CE.Resolve.RsSurface `RsAt`) and
-- its top-level surface (`Surface`). A request carries the facts the
-- measuring side has read (the walk's package.json, tsconfig.json and
-- Cargo.toml files and each Rust site's row up front); a rung that needs
-- one it lacks names it, and the reply's `tsWanted` lists every fact the
-- request's sites need next — the measuring side reads them and asks
-- again, as it does for the C family's response files. A fact is a
-- question of the file system or of a file's syntax tree only: which
-- paths and rows it asks about, in which order, and what an answer means
-- are this family's.
module CE.Resolve.TsFacts (
  Fact (..),
  Facts,
  Need,
  facts,
  wantedRow,
  both,
  textOf,
  isFile,
  isDir,
  nearestUp,
  nearestUpBy,
  tomlOf,
  payloadOf,
  firstTrue,
) where

import CE.Resolve.Json (readJsonc)
import CE.Resolve.Str (ancestors, joinDir)
import Data.Aeson (Value (..), toJSON)
import qualified Data.Aeson.Key as K
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | A question: a path's text (0), whether a path is a file (1), whether
-- `node_modules/<name>` under a directory is a directory (2), a path's
-- TOML document (3), a Rust file's answers at a row (4), a Rust file's
-- top-level surface (5).
data Fact = FText String | FFile String | FDir String String | FToml String | FRsAt String Int | FRsSurface String
  deriving (Eq, Ord, Show)

-- | The facts a request carries, each with its answer: a state (a text
-- or TOML fact: 0 not a file, 1 a file that does not read — as UTF-8, or
-- as TOML — 2 read; a file or directory fact: its truth; a Rust fact: 0
-- the file does not parse, 2 answered) and its document (a text fact's
-- read once as JSONC, the payload of the rest). `closed` facts answer
-- every unasked question "no" (the `inspect` questions, whose trees are
-- given whole).
data Facts = Facts
  { fxMap :: M.Map Fact (Int, Maybe Value)
  , fxClosed :: Bool
  }

-- | A result, or the facts it is waiting for.
type Need = Either (Set.Set Fact)

-- | The facts of `[op, a, b, state, payload | null]` rows (the contract
-- vouches for their shape: a text fact's payload is a string, a row
-- fact's `b` a row number).
facts :: Bool -> [(Int, String, String, Int, Maybe Value)] -> Facts
facts closed rows = Facts (M.fromList [(factOf op a b, (st, document op =<< t)) | (op, a, b, st, t) <- rows]) closed
 where
  document op t = case (op, t) of
    (0, String s) -> readJsonc (K.toString (K.fromText s))
    (0, _) -> Nothing
    (_, v) -> Just v

factOf :: Int -> String -> String -> Fact
factOf op a b = case op of
  0 -> FText a
  1 -> FFile a
  2 -> FDir a b
  3 -> FToml a
  4 -> FRsAt a (read b)
  _ -> FRsSurface a

-- | One wanted fact as its reply row, `[op, a, b]`.
wantedRow :: Fact -> Value
wantedRow f = toJSON $ case f of
  FText a -> (0 :: Int, a, "" :: String)
  FFile a -> (1, a, "")
  FDir a b -> (2, a, b)
  FToml a -> (3, a, "")
  FRsAt a row -> (4, a, show row)
  FRsSurface a -> (5, a, "")

-- | Two needs side by side: both results, or every fact either waits
-- for (neither waits on the other, so both are asked in one round).
both :: Need a -> Need b -> Need (a, b)
both (Right a) (Right b) = Right (a, b)
both a b = Left (waits a <> waits b)
 where
  waits :: Need x -> Set.Set Fact
  waits = either id (const Set.empty)

answer :: Facts -> Fact -> Maybe (Int, Maybe Value)
answer fx f = case M.lookup f (fxMap fx) of
  Nothing | fxClosed fx -> Just (0, Nothing)
  hit -> hit

-- | One fact's answer, or the fact itself as what is waited for.
asked :: Facts -> Fact -> Need (Int, Maybe Value)
asked fx f = maybe (Left (Set.singleton f)) Right (answer fx f)

-- | `read_jsonc`: the path's document, none when it is no readable file
-- or does not read as JSON.
textOf :: Facts -> String -> Need (Maybe Value)
textOf fx rel = snd <$> asked fx (FText rel)

isFile :: Facts -> String -> Need Bool
isFile fx rel = (== 1) . fst <$> asked fx (FFile rel)

isDir :: Facts -> String -> String -> Need Bool
isDir fx dir name = (== 1) . fst <$> asked fx (FDir dir name)

-- | `cargo::package`'s read: the path's TOML document, none when it is no
-- file or does not read as TOML.
tomlOf :: Facts -> String -> Need (Maybe Value)
tomlOf fx rel = snd <$> asked fx (FToml rel)

-- | A Rust fact's payload, none when the file does not parse.
payloadOf :: Facts -> Fact -> Need (Maybe Value)
payloadOf fx f = snd <$> asked fx f

-- | `roots::nearest_up`: the first of `dir` and its ancestors holding a
-- file `name`, probed by a text fact. Every unanswered probe before the
-- first answered hit is asked at once.
nearestUp :: Facts -> String -> String -> Need (Maybe String)
nearestUp = nearestUpBy FText

-- | `roots::nearest_up` probed by the fact `ask` makes of a candidate
-- path (a text fact for the TS configs, a TOML fact for Cargo.toml):
-- a file is one whose state is not 0.
nearestUpBy :: (String -> Fact) -> Facts -> String -> String -> Need (Maybe String)
nearestUpBy ask fx dir name = firstTrue [(joinDir d name, (>= 1) . fst <$> asked fx (ask (joinDir d name))) | d <- ancestors dir]

-- | The first candidate whose probe holds: unanswered probes before an
-- answered hit are all waited for; after it, none is asked.
firstTrue :: [(a, Need Bool)] -> Need (Maybe a)
firstTrue = go Set.empty
 where
  go waiting [] = if Set.null waiting then Right Nothing else Left waiting
  go waiting ((x, p) : rest) = case p of
    Right True -> if Set.null waiting then Right (Just x) else Left waiting
    Right False -> go waiting rest
    Left more -> go (waiting <> more) rest
