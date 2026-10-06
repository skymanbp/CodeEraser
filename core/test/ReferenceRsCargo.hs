{-# LANGUAGE OverloadedStrings #-}

-- | The Cargo half of ReferenceRs (plan v2.33 W2-text stage F), written
-- apart from CE.Resolve.Cargo: a case's manifest records as the decoded
-- TOML documents the core asks for, each record's package surface, the
-- crate roots, bin roots and lib root it holds among the walked files,
-- the files a package keeps private, and every fact the core names
-- answered from the case.
module ReferenceRsCargo (
  Pkg (..),
  pkgOf,
  manifests,
  owners,
  fact,
  ctxAt,
  libRootOf,
  crates,
  kept,
) where

import Data.Aeson (Value (..), object, toJSON, (.=))
import qualified Data.Aeson.Key as K
import Data.List (isSuffixOf, stripPrefix)
import Data.Maybe (isJust, listToMaybe, mapMaybe, maybeToList)
import qualified Data.Set as Set
import ReferenceResolveGen (splitOn)
import ReferenceRsGen
import ReferenceTs (ancestorsOf, joinRel, parent, within)

-- | The walked files' owning Cargo.toml files (the declared-target pass's
-- list), and each walked file with its owner (the mounts table's).
manifests :: RCase -> [String]
manifests c = Set.toList (Set.fromList (map snd (owners c)))

owners :: RCase -> [(String, String)]
owners c = [(f, m) | f <- rFiles c, Just m <- [nearest c (parent f)]]

-- | One wanted fact answered from the case: 0 none, 1 does not read, 2
-- the payload.
fact :: RCase -> (Int, String, String) -> Value
fact c (op, a, b) = toJSON [toJSON op, toJSON a, toJSON b, toJSON state, payload]
 where
  (state, payload) = case op of
    3 -> case manAt c a of
      Nothing -> (0 :: Int, Null)
      Just m | not (mReads m) -> (1, Null)
      Just m -> (2, document m)
    4 -> maybe (0, Null) (\its -> (2, atRow its (read b))) (lookup a (rSources c))
    _ -> maybe (0, Null) (\its -> (2, surfaceOf its)) (lookup a (rSources c))

-- | A manifest record as its decoded TOML document.
document :: Man -> Value
document m =
  object $
    ["package" .= object ["name" .= n | Just n <- [mName m]] | isJust (mName m)]
      <> ["lib" .= object ["path" .= l] | Just l <- [mLib m]]
      <> ["bin" .= [object ["name" .= ("b" :: String), "path" .= p] | p <- mBins m] | not (null (mBins m))]
      <> ["dependencies" .= object [K.fromString d .= ("1" :: String) | d <- mDeps m] | not (null (mDeps m))]

data Pkg = Pkg {pDir :: String, pName :: Maybe String, pLib :: Maybe String, pBins :: [String], pDeps :: [String]}

pkgOf :: Man -> Maybe Pkg
pkgOf m = if mReads m then Just (Pkg (mDir m) (mName m) (mLib m) (mBins m) (mDeps m)) else Nothing

manAt :: RCase -> String -> Maybe Man
manAt c p = listToMaybe [m | m <- rMans c, manPath m == p]

-- | The nearest Cargo.toml on disk, walking up from a directory.
nearest :: RCase -> String -> Maybe String
nearest c d = listToMaybe [p | a <- ancestorsOf d, let p = within a "Cargo.toml", isJust (manAt c p)]

-- | A directory's package (none when its nearest manifest does not read)
-- and its crate roots, the declared ones beside them.
ctxAt :: RCase -> String -> (Maybe Pkg, Set.Set String)
ctxAt c d = (pkg, maybe Set.empty (rootsOf c) pkg `Set.union` Set.fromList (rDeclared c))
 where
  pkg = nearest c d >>= manAt c >>= pkgOf

walked :: RCase -> [String] -> Set.Set String
walked c ps = Set.fromList [f | f <- ps, f `elem` rFiles c]

declaredOf :: Pkg -> [String] -> [String]
declaredOf p ts = mapMaybe (joinRel (pDir p)) ts

-- | The files directly under a conventional directory named `*.rs`, or
-- a `<name>/main.rs` one level down.
auto :: RCase -> Pkg -> String -> Set.Set String
auto c p sub = Set.fromList [f | f <- rFiles c, Just rest <- [stripPrefix (within (pDir p) sub <> "/") f], conventional (splitOn '/' rest)]
 where
  conventional segs = case segs of
    [leaf] -> ".rs" `isSuffixOf` leaf
    [_, "main.rs"] -> True
    _ -> False

rootsOf :: RCase -> Pkg -> Set.Set String
rootsOf c p = Set.unions (walked c (declaredOf p (maybeToList (pLib p) <> pBins p <> ["src/lib.rs", "src/main.rs", "build.rs"])) : map (auto c p) ["src/bin", "tests", "examples", "benches"])

binRootsOf :: RCase -> Pkg -> Set.Set String
binRootsOf c p = walked c (declaredOf p (pBins p <> ["src/main.rs"])) `Set.union` auto c p "src/bin"

libRootOf :: RCase -> Pkg -> Maybe String
libRootOf c p = listToMaybe (Set.toList (walked c (declaredOf p [maybe "src/lib.rs" id (pLib p)])))

crates :: RCase -> Set.Set String
crates c = Set.unions [rootsOf c p | m <- manifests c, Just p <- [manAt c m >>= pkgOf]]

-- | A named package without a lib target keeps every file private, one
-- with a lib target its bin roots alone.
kept :: RCase -> Set.Set String
kept c = Set.fromList [f | (f, m) <- owners c, Just p <- [manAt c m >>= pkgOf], isJust (pName p), keeps p f]
 where
  keeps p f = not (isJust (libRootOf c p)) || Set.member f (binRootsOf c p)
