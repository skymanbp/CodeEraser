-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The structure request's path form (plan v2.33 W1 item 3, additive
-- inside the unreleased 9.0.0; the port of cli/src/structure/rows.rs and
-- edges.rs at c2abca5d): the measuring side sends the walked judged paths,
-- the graph's measured file paths with their file-to-file arcs, the
-- declared layout, and — when their axes ride — the staleness facts and
-- the clone and dead files, all by path; this module builds the directory
-- tree (CE.Structure.Tree) and the dir-keyed tables the axes read, the
-- ones the measuring side sent as integers until then. A path the tree
-- cannot place is a fault answered by name (the message the measuring
-- side printed), never a guess.
module CE.Structure.Raw (Raw (..), Built (..), parseRaw, rawOffence, buildRaw, rustDebug) where

import CE.Structure.Tree (Dir (..), Tree (..), build, dirOf)
import Data.Aeson (Object, (.!=), (.:?))
import Data.Aeson.Types (Parser)
import Data.Char (GeneralCategory (..), generalCategory, ord)
import Data.List (dropWhileEnd, sort, sortOn)
import qualified Data.IntMap.Strict as IM
import qualified Data.Map.Strict as M
import qualified Data.Set as S
import Numeric (readHex, showHex)

-- | What the path form carries: the judged paths the tree is built over;
-- the measured graph file paths (slot order) and their arcs `[a, b]`; the
-- `[structure] layout` as `(path, weight)` in key order; per Markdown doc
-- with targets (path order) its newest window change and the newest
-- changes of its changed targets (target order), when `--days` rode; the
-- clone blocks' two files and the dead files, when `--deep` rode.
data Raw = Raw
  { rPaths :: [String]
  , rRefPaths :: [String]
  , rRefPairs :: [[Integer]]
  , rLayout :: [(String, Integer)]
  , rStale :: Maybe [(String, Integer, [Integer])]
  , rClones :: Maybe [(String, String)]
  , rDead :: Maybe [String]
  }

-- | The tables built from it, as the integer form names them; the
-- reference split and the crossing dir edges (`fileRefs`, `dirEdges`) and
-- the staleness rows (`staleDocRows`, `staleEdgeRows`) as the pairs one
-- pass builds.
data Built = Built
  { bTree :: Tree
  , bNodes :: [[Integer]]
  , bShapes :: [[Integer]]
  , bConventions :: [[Integer]]
  , bRefs :: ([[Integer]], [[Integer]])
  , bDeclared :: [[Integer]]
  , bStale :: Maybe ([[Integer]], [[Integer]])
  , bRedundancy :: Maybe [[Integer]]
  }

-- | The path form, when the request carries `paths`.
parseRaw :: Object -> Parser (Maybe Raw)
parseRaw o = do
  paths <- o .:? "paths"
  case paths of
    Nothing -> pure Nothing
    Just ps ->
      fmap Just $
        Raw ps
          <$> o .:? "refPaths" .!= []
          <*> o .:? "refPairs" .!= []
          <*> o .:? "layout" .!= []
          <*> o .:? "staleDocs"
          <*> o .:? "clonePairs"
          <*> o .:? "deadPaths"

-- | The path form's own contract: every arc a pair of slots of
-- `refPaths`. (The built tables then meet the integer form's checks.)
rawOffence :: Raw -> Maybe String
rawOffence raw = case [i | (i, row) <- zip [0 :: Int ..] (rRefPairs raw), not (slotPair row)] of
  i : _ -> Just ("refPairs " <> show i <> ": need [a,b], both slots of refPaths")
  [] -> Nothing
 where
  n = toInteger (length (rRefPaths raw))
  slotPair row = case row of
    [a, b] -> all (\x -> x >= 0 && x < n) [a, b]
    _ -> False

-- | The tables, or the first path the tree cannot place, in the order the
-- measuring side met them: the staleness docs, the clone files, the dead
-- files, the arcs' files, the layout. The arcs must name slots of
-- `refPaths` (a malformed pair is the caller's to refuse first).
buildRaw :: Raw -> Either String Built
buildRaw raw = do
  stale <- traverse (staleTables t) (rStale raw)
  dup <- traverse (dupByDir t) (rClones raw)
  dead <- traverse (deadByDir t) (rDead raw)
  refs <- refTables t (rRefPaths raw) (rRefPairs raw)
  declared <- declaredRows t (rLayout raw)
  pure
    Built
      { bTree = t
      , bNodes = [[toInteger i, toInteger (dParent d), toInteger (dDepth d), toInteger (dSubdirs d), toInteger (dFiles d)] | (i, d) <- dirs]
      , bShapes = [[toInteger i, toInteger bits, toInteger n] | (i, d) <- dirs, (bits, n) <- M.toAscList (dShapes d)]
      , bConventions = [[toInteger i, toInteger (dConventions d)] | (i, d) <- dirs, dConventions d > 0]
      , bRefs = refs
      , bDeclared = declared
      , bStale = stale
      , bRedundancy = redundancy <$> dup <*> dead
      }
 where
  t = build (rPaths raw)
  dirs = IM.toAscList (tDirs t)

-- | A file's directory, or the measuring side's message naming it.
placed :: Tree -> String -> String -> Either String Int
placed t what path = maybe (Left (what <> " " <> path <> " outside the walked tree")) Right (dirOf t path)

-- | `ref_rows` (with `file_join` and edges.rs's `aggregate` / `directed`):
-- each measured file's directory, then `[dirId, inside, outside, count]`
-- — how many files share that participation — and the directed crossing
-- table `[fromDir, toDir, count]`, both off the one arc list.
refTables :: Tree -> [String] -> [[Integer]] -> Either String ([[Integer]], [[Integer]])
refTables t paths pairs = do
  fileDirs <- traverse (placed t "graph node") paths
  let slots = IM.fromList (zip [0 ..] fileDirs)
      arcs = [(fromInteger a, fromInteger b) | [a, b] <- pairs]
      dirAt s = slots IM.! s
      side (a, b) = if dirAt a /= dirAt b then (0, 1) else (1, 0)
      bumps = IM.fromListWith add (concat [[(a, side p), (b, side p)] | p@(a, b) <- arcs])
      add (x, y) (u, v) = (x + u, y + v) :: (Integer, Integer)
      participation = [(dirAt s, io) | s <- IM.keys slots, Just io@(i, o) <- [IM.lookup s bumps], i + o > 0]
      counted = M.fromListWith (+) [((toInteger d, i, o), 1 :: Integer) | (d, (i, o)) <- participation]
      crossing = M.fromListWith (+) [((dirAt a, dirAt b), 1 :: Integer) | (a, b) <- arcs, dirAt a /= dirAt b]
  pure
    ( [[d, i, o, n] | ((d, i, o), n) <- M.toAscList counted]
    , [[toInteger a, toInteger b, n] | ((a, b), n) <- M.toAscList crossing]
    )

-- | `declared_rows`: each layout path (trailing '/' trimmed, "." the root)
-- to `[dirId, weight]`, ascending; a path naming no walked directory is
-- the measuring side's config error, spelled as it printed it.
declaredRows :: Tree -> [(String, Integer)] -> Either String [[Integer]]
declaredRows t layout = fmap sort (traverse row layout)
 where
  row (path, w) =
    let trimmed = dropWhileEnd (== '/') path
        key = if trimmed == "." then "" else trimmed
     in case M.lookup key (tIds t) of
          Just d -> Right [toInteger d, w]
          Nothing -> Left ("[structure] layout declares " <> rustDebug path <> ", which is not a walked directory")

-- | `stale_doc_rows`' table half: the docs ordered by (dirId, path) as
-- `[dirId, docTs]` (doc identity = row index), and `[docIdx, targetTs]`
-- per changed target.
staleTables :: Tree -> [(String, Integer, [Integer])] -> Either String ([[Integer]], [[Integer]])
staleTables t docs = do
  placedDocs <- traverse (\(md, ts, tts) -> fmap (\d -> (d, md, ts, tts)) (placed t "md node" md)) docs
  let byDir = sortOn (\(d, md, _, _) -> (d, md)) placedDocs
  pure
    ( [[toInteger d, ts] | (d, _, ts, _) <- byDir]
    , [[i, tt] | (i, (_, _, _, tts)) <- zip [0 ..] byDir, tt <- tts]
    )

-- | `redundancy_rows`' clone half: a block counts once per directory it
-- touches.
dupByDir :: Tree -> [(String, String)] -> Either String (M.Map Int Integer)
dupByDir t blocks = do
  touched <- traverse (\(a, b) -> (\da db -> S.fromList [da, db]) <$> placed t "clone block file" a <*> placed t "clone block file" b) blocks
  pure (M.fromListWith (+) [(d, 1) | ds <- touched, d <- S.toList ds])

-- | `redundancy_rows`' dead half: dead files per directory.
deadByDir :: Tree -> [String] -> Either String (M.Map Int Integer)
deadByDir t dead = M.fromListWith (+) . map (\d -> (d, 1)) <$> traverse (placed t "dead node") dead

-- | `[dirId, dupBlocks, deadUnits]` over every directory either touches.
redundancy :: M.Map Int Integer -> M.Map Int Integer -> [[Integer]]
redundancy dup dead =
  [ [toInteger d, M.findWithDefault 0 d dup, M.findWithDefault 0 d dead]
  | d <- S.toAscList (M.keysSet dup <> M.keysSet dead)
  ]

-- | Rust's `{:?}` of a `str`: quoted, `\0 \t \r \n \\ \"` escaped, a
-- grapheme-extending char or a char outside Rust's printable set as
-- `\u{hex}`. Printable follows core::unicode::printable's generator (not
-- Cc Cf Cs Co Cn Zl Zp Zs, the space excepted) over GHC's Unicode table;
-- grapheme-extending = Mn, Me and `otherExtend`. Rust's Unicode table is
-- the one that counts (the Chars / Lower posture of CE.Similar):
-- `otherExtend` is the set of code points rustc 1.94.1's `{:?}` escapes
-- that GHC 9.14.1's Mn / Me and the printable rule do not already cover,
-- measured over every code point (probe
-- D:/Projects/.worktrees/CodeEraser/w1i3_review_probe/, rs.txt against
-- hs_lf.txt); outside it the two sides agreed on all 954,527 other
-- escaped code points at this pairing, and with it this function escapes
-- exactly the 954,588 code points rustc 1.94.1 does, each spelled alike.
-- A toolchain move re-measures it.
rustDebug :: String -> String
rustDebug s = "\"" <> concatMap esc s <> "\""
 where
  esc c = case c of
    '\0' -> "\\0"
    '\t' -> "\\t"
    '\r' -> "\\r"
    '\n' -> "\\n"
    '\\' -> "\\\\"
    '"' -> "\\\""
    _
      | extending c || not (printable c) -> "\\u{" <> showHex (ord c) "}"
      | otherwise -> [c]
  printable c = c == ' ' || generalCategory c `notElem` [Control, Format, Surrogate, PrivateUse, NotAssigned, LineSeparator, ParagraphSeparator, Space]
  extending c = generalCategory c `elem` [NonSpacingMark, EnclosingMark] || any (\(lo, hi) -> ord c >= lo && ord c <= hi) otherExtend
  otherExtend = map hexRange (words otherExtendText)
  hexRange w = case break (== '-') w of
    (lo, '-' : hi) -> (hex lo, hex hi)
    (one, _) -> (hex one, hex one)
  hex h = case readHex h of
    [(n, "")] -> n
    _ -> error ("rustDebug: bad hex " <> h)

-- | `otherExtend` as hex code points and inclusive ranges (see rustDebug).
otherExtendText :: String
otherExtendText =
  "9be 9d7 b3e b57 bbe bd7 cc0 cc2 cc7-cc8 cca-ccb cd5-cd6 d3e d57 dcf ddf 1715 1734 1b35 1b3b 1b3d \
  \1b43-1b44 1baa 1bf2-1bf3 200c 302e-302f a953 a9c0 ff9e-ff9f 111c0 11235 1133e 1134d 11357 113b8 \
  \113c2 113c5 113c7-113c9 113cf 114b0 114bd 115af 116b6 11930 1193d 11f41 16ff0-16ff1 1d165-1d166 \
  \1d16d-1d172 e0020-e007f"
