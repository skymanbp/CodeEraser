-- | The tree one resolve.request describes, indexed once (plan v2.33
-- wave W2a): the directory and file tables as paths of segment ids,
-- the vocabulary's ids, the affix table, and the joins every ladder
-- shares. A path is a list of pieces; a piece is a segment id or the
-- one thing the core cannot name — a concatenation (`prefix ++ affix`)
-- the tree spells nowhere, which matches no file and is popped like
-- any other piece by a later `..`.
--
-- The join is the path join the measuring side's ladders used
-- (`roots::join_rel`): the directory's pieces are literal (its empty
-- pieces dropped), the spec's pieces are read — an empty piece and `.`
-- stay put, `..` climbs and climbing above the root is no path.
module CE.Resolve.World (
  Pc (..),
  World (..),
  world,
  word,
  affix,
  spelled,
  isWord,
  joinRel,
  fileAt,
  inScope,
  parentOf,
  glue,
  endsWith,
  rawDir,
  known,
  hits,
) where

import CE.Resolve.Request (ResolveReq (..))
import CE.Resolve.Vocab (Affix (..), Word' (..), affixAt, codeOf, wordAt)
import Data.Array (Array, listArray, (!))
import qualified Data.IntSet as IS
import qualified Data.Map.Strict as M
import qualified Data.Set as Set

-- | One piece of a path: a segment, or a concatenation nothing in the
-- tree spells.
data Pc = K !Int | U
  deriving (Eq, Ord, Show)

data World = World
  { wVocab :: Array Int Int
  , wAffix :: M.Map (Int, Int) Int
  , wEnds :: Set.Set (Int, Int)
  , wDirPath :: Array Int [Int]
  , wDirAt :: M.Map [Int] Int
  , wFileDir :: Array Int Int
  , wFileBase :: Array Int Int
  , wFileLang :: Array Int Integer
  , wFileAt :: M.Map [Int] Int
  , wFileCount :: Int
  , wWalked :: Array Int Bool
  , wGoDirs :: IS.IntSet
  }

-- | The world of a request its contract already passed (every id in
-- range, every parent before its child).
world :: ResolveReq -> World
world rq = w
 where
  w =
    World
      { wVocab = arr (map fromInteger (rqVocab rq))
      , wAffix = M.fromList [((i a, i p), i x) | [a, p, x] <- rqAffixes rq]
      , wEnds = Set.fromList [(i a, i x) | [a, _, x] <- rqAffixes rq]
      , wDirPath = dirPath
      , wDirAt = M.fromList (zip (elemsOf dirPath (nDirs + 1)) [0 ..])
      , wFileDir = arr [i d | (d : _) <- files]
      , wFileBase = arr [i b | (_ : b : _) <- files]
      , wFileLang = arr [l | (_ : _ : l : _) <- files]
      , wFileAt = M.fromList [(dirPath ! i d <> [i b], f) | (f, d : b : _ : 1 : _) <- zip [0 ..] files]
      , wFileCount = length files
      , wWalked = arr [x == 1 | (_ : _ : _ : x : _) <- files]
      , wGoDirs = IS.fromList [i d | (d : b : _ : 1 : _) <- files, goSource w (i b)]
      }
  files = rqFiles rq
  nDirs = length (rqDirs rq)
  dirPath = listArray (0, nDirs) ([] : [dirPath ! i p <> [i s] | [p, s] <- rqDirs rq])
  arr xs = listArray (0, length xs - 1) xs
  elemsOf a n = [a ! k | k <- [0 .. n - 1]]
  i = fromInteger

-- | A directly held importable Go file: a `.go` name that is not a
-- `_test.go` one.
goSource :: World -> Int -> Bool
goSource w base = endsWith w AGo base && not (endsWith w AGoTest base)

word :: World -> Word' -> Int
word w x = wVocab w ! wordAt x

affix :: World -> Affix -> Int
affix w a = wVocab w ! affixAt a

-- | A standard-library entry's pieces as the request's segments.
spelled :: World -> [String] -> [Int]
spelled w = map ((wVocab w !) . codeOf)

-- | Whether a segment is one fixed word.
isWord :: World -> Word' -> Int -> Bool
isWord w x s = s == word w x

-- | `prefix ++ affix`: the segment the tree spells so, or the piece
-- that names nothing.
glue :: World -> Int -> Pc -> Pc
glue w a (K p) = maybe U K (M.lookup (a, p) (wAffix w))
glue _ _ U = U

-- | Whether a segment ends with an affix (the affix table's rows).
endsWith :: World -> Affix -> Int -> Bool
endsWith w a s = Set.member (affix w a, s) (wEnds w)

-- | A raw directory as the join reads it: its empty pieces dropped.
rawDir :: World -> [Int] -> [Int]
rawDir w = filter (not . isWord w WEmpty)

-- | The join (module header): directory literal, spec read.
joinRel :: World -> [Int] -> [Pc] -> Maybe [Pc]
joinRel w dir = fmap reverse . foldl step (Just (reverse (map K (rawDir w dir))))
 where
  step acc p = acc >>= move p
  move (K s) st
    | isWord w WEmpty s || isWord w WDot s = Just st
    | isWord w WDotDot s = case st of
        [] -> Nothing
        (_ : rest) -> Just rest
  move p st = Just (p : st)

-- | The walked file at a path (an unwalked file — a site's own file
-- the walk did not hold — is never a target).
fileAt :: World -> [Pc] -> Maybe Int
fileAt w ps = traverse known ps >>= (`M.lookup` wFileAt w)

known :: Pc -> Maybe Int
known (K s) = Just s
known U = Nothing

-- | The walked file a directory holds under a spec.
inScope :: World -> [Int] -> [Pc] -> Maybe Int
inScope w dir spec = joinRel w dir spec >>= fileAt w

-- | A file's directory, as pieces.
parentOf :: World -> Int -> [Int]
parentOf w f = wDirPath w ! (wFileDir w ! f)

-- | The distinct files of a rung (ascending ids, which the measuring
-- side assigns in path order).
hits :: [Maybe Int] -> [Int]
hits = IS.toList . IS.fromList . concatMap (maybe [] pure)
