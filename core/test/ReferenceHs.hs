-- | An independently written reference for the Haskell rungs and the
-- cabal facts (plan v2.33 W2-text stage D), written apart from
-- CE.Resolve.Hs, CE.Resolve.Cabal and CE.Resolve.CabalWalk: the cases
-- (ReferenceHsGen) are read back from their records, never from the text
-- the core parses, so the reader is on the core's side only; the rungs
-- are set folds over the records. Every site of the two hundred cases
-- must get the same answer from both, the declared mains and the private
-- files must be the same sets, and the cases reach every answer the
-- Haskell rungs give.
module ReferenceHs (equivalence) where

import CE.Resolve (respond)
import CE.Resolve.Cost (Reason (..))
import CE.Resolve.Tables (hsBoot)
import Data.Aeson (Value (..), decodeStrict, encode)
import qualified Data.Aeson.Key as K
import qualified Data.Aeson.KeyMap as KM
import Data.Aeson.Types (parseJSON, parseMaybe)
import qualified Data.ByteString.Lazy as BL
import Data.Char (isAsciiUpper, toLower)
import Data.List (isPrefixOf, stripPrefix)
import Data.Maybe (mapMaybe)
import qualified Data.Set as Set
import ReferenceHsGen
import ReferenceResolveGen (Ref (..), answers, refShape, splitOn)
import WireHarness (battery, runChecks)

equivalence :: IO Bool
equivalence =
  runChecks
    ( battery
        ("resolve: 200 Haskell cases, 2400 sites, shipped = reference (rungs, mains, private)", "resolve: the Haskell cases reach every answer of the Haskell rungs")
        (map disagree hsCases)
        ["file 1", "file 2", "external 3", "OutOfScope", "AmbiguousRoot", "AmbiguousWorkspace"]
        (Set.fromList [refShape (refSite c s) | c <- hsCases, s <- hSites c])
    )

-- | The first disagreement of a case, spelled.
disagree :: HCase -> Maybe String
disagree c = case reply of
  Nothing -> Just "no reply"
  Just v
    | answers v /= Just want -> Just ("sites: core " <> show (answers v) <> " reference " <> show want)
    | strings v "mains" /= Just (Set.toList (mains c)) -> Just ("mains: core " <> show (strings v "mains"))
    | strings v "private" /= Just (private c) -> Just ("private: core " <> show (strings v "private"))
    | otherwise -> Nothing
 where
  reply = either (const Nothing) decodeStrict (respond "9.0.0" (BL.toStrict (encode (hsRequest c))))
  want = map (refSite c) (hSites c)
  strings :: Value -> String -> Maybe [String]
  strings v k = case v of
    Object o -> KM.lookup (K.fromString k) o >>= parseMaybe parseJSON
    _ -> Nothing

-- | A cabal as the rungs read it: its directory, name ("" when none),
-- whether a bare library exists, its exposed and hidden modules, the
-- build-depends names, each stanza's roots (the package directory when it
-- names none), the library stanzas' roots, the mains.
data Pkg = Pkg
  { pDir :: String
  , pName :: String
  , pLibrary :: Bool
  , pExposed :: Set.Set String
  , pHidden :: Set.Set String
  , pDeps :: [String]
  , pRoots :: [[String]]
  , pLibRoots :: [String]
  , pMains :: [(Maybe String, [String])]
  }

pkg :: Cabal -> Pkg
pkg (Cabal d _ nm stanzas _) =
  Pkg
    { pDir = d
    , pName = maybe "" id nm
    , pLibrary = any bare stanzas
    , pExposed = exposed
    , pHidden = Set.fromList (concat [o | Stanza _ _ _ o _ _ <- stanzas]) `Set.difference` exposed
    , pDeps = concat [ds | Stanza _ _ _ _ ds _ <- stanzas]
    , pRoots = map rootsOf stanzas
    , pLibRoots = concat [rootsOf s | s <- stanzas, bare s]
    , pMains = [(m, rootsOf s) | s@(Stanza _ _ _ _ _ m) <- stanzas]
    }
 where
  exposed = Set.fromList (concat [e | Stanza _ _ e _ _ _ <- stanzas])
  bare (Stanza h _ _ _ _ _) = map toLower h == "library"
  rootsOf (Stanza _ rs _ _ _ _) = if null rs then [d] else map (under d) rs
  under base r = case (base, r) of
    (_, ".") -> base
    ("", _) -> r
    _ -> base <> "/" <> r

pkgs :: HCase -> [Pkg]
pkgs = map pkg . hCabals

-- | One site.
refSite :: HCase -> (String, String) -> Ref
refSite c (from, written)
  | not (shaped modl) = RUnres OutOfScope
  | otherwise = case owning of
      Left why -> RUnres why
      Right o -> case (r1 o, r2 o) of
        (Just a, _) -> a
        (_, Just a) -> a
        _ -> r3 o
 where
  (named, modl) = case written of
    '"' : rest | (p, '"' : m) <- break (== '"') rest -> (Just p, dropWhile (== ' ') m)
    _ -> (Nothing, written)
  rel = map (\ch -> if ch == '.' then '/' else ch) modl <> ".hs"
  walked = Set.fromList (hFiles c)
  inTree root = let p = if null root then rel else root <> "/" <> rel in [p | Set.member p walked]
  holders = [p | p <- pkgs c, null (pDir p) || (pDir p <> "/") `isPrefixOf` from]
  deepest = maximum (0 : map (length . pDir) holders)
  owning = case [p | p <- holders, length (pDir p) == deepest] of
    [] -> Right Nothing
    [p] -> Right (Just p)
    _ -> Left AmbiguousWorkspace
  r1 o
    | maybe False (\p -> maybe True ((/= p) . pName) o) named = Nothing
    | otherwise = case Set.toList (Set.fromList (concatMap inTree (rootsHolding o))) of
        [] -> Nothing
        [p] -> Just (RFile p 1)
        _ -> Just (RUnres AmbiguousRoot)
  rootsHolding Nothing = [""]
  rootsHolding (Just p) =
    let holding = [rs | rs <- pRoots p, any (\r -> null r || (r <> "/") `isPrefixOf` from) rs]
     in concat (if null holding then pRoots p else holding)
  r2 o = case [p | p <- pkgs c, not (null (pName p)), maybe True ((/= pDir p) . pDir) o, wanted o p, pLibrary p, Set.member modl (pExposed p)] of
    [] -> Nothing
    [p] -> Just $ case Set.toList (Set.fromList (concatMap inTree (pLibRoots p))) of
      [f] -> RFile f 2
      [] -> RUnres OutOfScope
      _ -> RUnres AmbiguousRoot
    _ -> Just (RUnres AmbiguousWorkspace)
  wanted o p = maybe (maybe False ((pName p `elem`) . pDeps) o) (== pName p) named
  r3 o = if any (visible o) [p | (p, ms) <- hsBoot, modl `elem` ms] then RExt 3 else RUnres OutOfScope
  visible o p = maybe True (== p) named && maybe True ((p `elem`) . pDeps) o

shaped :: String -> Bool
shaped m = not (null m) && all (\s -> take 1 s /= "" && all isAsciiUpper (take 1 s)) (splitOn '.' m)

-- | The walked mains every cabal declares.
mains :: HCase -> Set.Set String
mains c = Set.fromList [f | p <- pkgs c, (Just m, rs) <- pMains p, r <- rs, let f = if null r then m else r <> "/" <> m, f `elem` hFiles c]

-- | The owned files their cabal keeps private.
private :: HCase -> [String]
private c = [f | (f, path) <- owners c, Just p <- [lookup path [(cabalPath cb, pkg cb) | cb <- hCabals c]], keeps p f]
 where
  keeps p f = not (pLibrary p) || any (\m -> Set.member m (pHidden p)) (mapMaybe (spell f) (concat (pRoots p)))
  spell f r = do
    below <- if null r then Just f else stripPrefix (r <> "/") f
    stem <- reverse <$> stripPrefix (reverse ".hs") (reverse below)
    Just (map (\ch -> if ch == '/' then '.' else ch) stem)
