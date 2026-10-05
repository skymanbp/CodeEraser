{-# LANGUAGE OverloadedStrings #-}

-- | What the TS rungs need of the file system (plan v2.33 W2-text stage
-- E), asked of the measuring side as facts instead of read: whether a
-- configuration path is a file and its text (`nearest_up`'s probe, then
-- `read_jsonc`), whether a compiled JavaScript twin is a file
-- (`esm_rewrite`), whether `node_modules/<name>` under a directory is a
-- directory (`bare_rung`). A request carries the facts the measuring
-- side has read (the walk's package.json and tsconfig.json files
-- up front); a rung that needs one it lacks names it, and the reply's
-- `tsWanted` lists every fact the request's sites need next — the
-- measuring side reads them and asks again, as it does for the C
-- family's response files. A fact is a question of the file system
-- only: which paths it asks about, in which order, and what an answer
-- means are this family's.
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
  firstTrue,
) where

import CE.Resolve.Json (readJsonc)
import CE.Resolve.Str (ancestors, joinDir)
import Data.Aeson (Value, toJSON)
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | A question: a path's text (0), whether a path is a file (1),
-- whether `node_modules/<name>` under a directory is a directory (2).
data Fact = FText String | FFile String | FDir String String
  deriving (Eq, Ord, Show)

-- | The facts a request carries, each with its answer: a text fact's
-- state (0 not a file, 1 a file that does not read as UTF-8 text, 2 its
-- text) and its JSONC document read once; a file or directory fact's
-- truth. `closed` facts answer every unasked question "no" (the
-- `inspect` questions, whose trees are given whole).
data Facts = Facts
  { fxMap :: M.Map Fact (Int, Maybe Value)
  , fxClosed :: Bool
  }

-- | A result, or the facts it is waiting for.
type Need = Either (Set.Set Fact)

-- | The facts of `[op, a, b, state, text | null]` rows (the contract
-- vouches for their shape).
facts :: Bool -> [(Int, String, String, Int, Maybe String)] -> Facts
facts closed rows = Facts (M.fromList [(factOf op a b, (st, readJsonc =<< t)) | (op, a, b, st, t) <- rows]) closed

factOf :: Int -> String -> String -> Fact
factOf op a b = case op of
  0 -> FText a
  1 -> FFile a
  _ -> FDir a b

-- | One wanted fact as its reply row, `[op, a, b]`.
wantedRow :: Fact -> Value
wantedRow f = toJSON $ case f of
  FText a -> (0 :: Int, a, "" :: String)
  FFile a -> (1, a, "")
  FDir a b -> (2, a, b)

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

-- | `read_jsonc`: the path's document, none when it is no readable file
-- or does not read as JSON.
textOf :: Facts -> String -> Need (Maybe Value)
textOf fx rel = maybe (Left (Set.singleton (FText rel))) (Right . snd) (answer fx (FText rel))

isFile :: Facts -> String -> Need Bool
isFile fx rel = maybe (Left (Set.singleton (FFile rel))) (Right . (== 1) . fst) (answer fx (FFile rel))

isDir :: Facts -> String -> String -> Need Bool
isDir fx dir name = maybe (Left (Set.singleton (FDir dir name))) (Right . (== 1) . fst) (answer fx (FDir dir name))

-- | `roots::nearest_up`: the first of `dir` and its ancestors holding a
-- file `name`. Every unanswered probe before the first answered hit is
-- asked at once.
nearestUp :: Facts -> String -> String -> Need (Maybe String)
nearestUp fx dir name = firstTrue [(joinDir d name, probe d) | d <- ancestors dir]
 where
  probe d = maybe (Left (Set.singleton (FText (joinDir d name)))) (Right . (>= 1) . fst) (answer fx (FText (joinDir d name)))

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
