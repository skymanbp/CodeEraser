-- | The resolve.request boundary contract (plan v2.33 wave W2a): what a
-- well-formed request IS — every id in its table's range, every
-- directory after its parent, no path twice, each row the width its
-- table names and each specifier the token shape its language reads —
-- and the cap every table counts against together. The first offender
-- in table order is refused by name ("<table> <i>: <why>"); a request
-- that passes is the one CE.Resolve.World indexes without a check of
-- its own.
module CE.Resolve.Contract (offence, overCap) where

import CE.Resolve.Cost
import CE.Resolve.Request
import CE.Resolve.Vocab (kindRequire, vocabLength)
import Data.Foldable (asum)
import qualified Data.Set as Set

-- | All the tables' rows together against one cap.
overCap :: ResolveReq -> Bool
overCap rq = toInteger (sum (map (length . snd) (tableRows rq))) > resolveCap

-- | The first offender, table by table.
offence :: ResolveReq -> Maybe String
offence rq =
  asum
    [ if rqSegs rq < 0 then Just "segs: negative count" else Nothing
    , vocab
    , asum [rows name (shape b name) t | (name, t) <- tableRows rq]
    , unique "dir" [(p, s) | [p, s] <- rqDirs rq]
    , unique "file" [(d, f) | (d : f : _) <- rqFiles rq]
    , asum (zipWith (\i d -> if seg b d then Nothing else Just ("py.dep " <> show i <> ": segment out of range")) [0 :: Int ..] (pyDeps (rqPy rq)))
    ]
 where
  b = boundsOf rq
  vocab
    | length (rqVocab rq) /= vocabLength =
        Just ("vocab: " <> show (length (rqVocab rq)) <> " ids for " <> show vocabLength <> " words")
    | not (segs b (rqVocab rq)) = Just "vocab: segment out of range"
    | otherwise = Nothing

-- | The ranges a request's ids are checked against: the segment count
-- and the directory, file, chain and module table lengths.
data Bounds = Bounds {bSegs, bDirs, bFiles, bChains, bMods :: Integer}

boundsOf :: ResolveReq -> Bounds
boundsOf rq =
  Bounds
    { bSegs = rqSegs rq
    , bDirs = toInteger (length (rqDirs rq))
    , bFiles = toInteger (length (rqFiles rq))
    , bChains = toInteger (length (cChains (rqC rq)))
    , bMods = toInteger (length (goMods (rqGo rq)))
    }

seg, file, chain :: Bounds -> Integer -> Bool
seg b x = x >= 0 && x < bSegs b
file b x = x >= 0 && x < bFiles b
chain b x = x >= 0 && x < bChains b

segs :: Bounds -> [Integer] -> Bool
segs b = all (seg b)

-- | One row of a named table: its own shape, else segments in range.
shape :: Bounds -> String -> Int -> [Integer] -> Maybe String
shape b name i row = maybe (if segs b row then Nothing else Just "segment out of range") (\f -> f i row) (lookup name (shapes b))

shapes :: Bounds -> [(String, Int -> [Integer] -> Maybe String)]
shapes b =
  [ ("affix", need "malformed affix (need [affix,prefix,whole] segments)" (\r -> length r == 3 && segs b r))
  , ("dir", \i -> need "malformed dir (need [parent,segment], parent before the dir)" (dirRow b i) i)
  , ("file", need "malformed file (need [dir,basename,lang,walked])" (fileRow b))
  , ("site", \_ -> siteRow b)
  , ("lua.template", need "malformed template (need [head,n,dir…,tail…])" (templateRow b))
  , ("go.mod", need "malformed module (need [n,dir…,path…])" (modRow b))
  , ("go.replace", need "malformed replace (need [mod,form,n,old…,new…])" (replaceRow b))
  , ("c.chain", need "malformed chain (need [msvc,ownDir])" (\r -> length r == 2 && all (`elem` [0, 1]) r))
  , ("c.search", need "malformed search (need [chain,class,kind,dir…])" (searchRow b))
  , ("c.forced", need "malformed forced include (need [chain,how,piece…])" (forcedRow b))
  , ("c.seat", need "malformed seat (need [file,chain,placed,dir…])" (seatRow b))
  , ("c.flags", need "malformed flags directory (need [chain,dir…])" (flagsRow b))
  , ("c.include", need "malformed include (need [file,system,piece…])" (includeRow b))
  ]
 where
  need msg ok _ row = if ok row then Nothing else Just msg

-- | The fixed-width and leading-column rows.
dirRow :: Bounds -> Int -> [Integer] -> Bool
dirRow b i r = case r of
  [p, s'] -> p >= 0 && p <= toInteger i && seg b s'
  _ -> False

fileRow, templateRow, modRow, replaceRow :: Bounds -> [Integer] -> Bool
fileRow b r = case r of
  [d, f, l, x] -> d >= 0 && d <= bDirs b && seg b f && l >= 0 && x `elem` [0, 1]
  _ -> False
templateRow b r = case r of
  h : k : rest -> seg b h && k >= 0 && k <= toInteger (length rest) && segs b rest
  _ -> False
modRow b r = case r of
  k : rest -> k >= 0 && k < toInteger (length rest) && segs b rest
  _ -> False
replaceRow b r = case r of
  m : f : k : rest -> m >= 0 && m < bMods b && f >= 0 && f <= 3 && k > 0 && k < toInteger (length rest) && segs b rest
  _ -> False

-- | The C tables' rows.
searchRow, forcedRow, seatRow, flagsRow, includeRow :: Bounds -> [Integer] -> Bool
searchRow b r = case r of
  c : cls : kind : d -> chain b c && cls `elem` [0, 1, 2] && kind `elem` [0, 1] && segs b d
  _ -> False
forcedRow b r = case r of
  c : how : p -> chain b c && forcedShape b how p
  _ -> False
seatRow b r = case r of
  f : c : placed : d -> file b f && chain b c && placed `elem` [0, 1] && (placed == 1 || null d) && segs b d
  _ -> False
flagsRow b r = case r of
  c : d -> chain b c && segs b d
  _ -> False
includeRow b r = case r of
  f : s' : p -> file b f && s' `elem` [0, 1] && not (null p) && segs b p
  _ -> False

forcedShape :: Bounds -> Integer -> [Integer] -> Bool
forcedShape b how p
  | how == forcedOutside = null p
  | how == forcedRelative = not (null p) && segs b p
  | otherwise = how == forcedPlaced && segs b p

-- | A site row: language, kind, file, form, then the specifier's tokens
-- in the shape its language reads.
siteRow :: Bounds -> [Integer] -> Maybe String
siteRow b r = case r of
  lang : kind : from : form : toks -> site lang kind from form toks
  _ -> Just "malformed site (need [lang,kind,from,form,token…])"
 where
  site lang kind from form toks
    | lang `notElem` resolvedLangs = Just "language this family does not resolve"
    | kind < 0 || not (file b from) || form < 0 = Just "kind, file or form out of range"
    | lang == langPy = dotted toks
    | lang == langLua && kind == kindRequire = alternating toks
    | otherwise = if not (null toks) && segs b toks then Nothing else Just "malformed path (need one segment or more)"
  dotted toks
    | null toks = Nothing
    | all (\t -> seg b t || t == sepDot) toks && take 1 toks /= [sepDot] && take 1 (reverse toks) /= [sepDot] && notTwice toks = Nothing
    | otherwise = Just "malformed dotted name"
  notTwice ts = and (zipWith (\x y -> not (x == sepDot && y == sepDot)) ts (drop 1 ts))
  alternating toks
    | odd (length toks) && and (zipWith piece [0 :: Int ..] toks) = Nothing
    | otherwise = Just "malformed module name (need piece, separator, piece…)"
  piece k t = if even k then seg b t else t == sepDot || t == sepSlash

-- | Each table's rows through its shape, the first offender named.
rows :: String -> (Int -> [Integer] -> Maybe String) -> [[Integer]] -> Maybe String
rows name f = asum . zipWith (\i r -> fmap (\why -> name <> " " <> show i <> ": " <> why) (f i r)) [0 ..]

-- | No two rows of a table name one path.
unique :: (Ord a) => String -> [a] -> Maybe String
unique name keys = go Set.empty (zip [0 :: Int ..] keys)
 where
  go _ [] = Nothing
  go seen ((i, k) : rest)
    | Set.member k seen = Just (name <> " " <> show i <> ": a path named twice")
    | otherwise = go (Set.insert k seen) rest
