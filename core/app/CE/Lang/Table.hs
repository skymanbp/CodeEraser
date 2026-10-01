{-# LANGUAGE DataKinds #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE UndecidableInstances #-}

-- | How a definition document is read (plan v2.32 step 1; design
-- booklet docs/reference/authority-track.md §4.2): a record by its
-- field names, every absent field its blank ('Row'); a choice by its
-- constructor's name ('Tag'); and the one table too long to quote name
-- by name, the GHC global package table, as rows of words ('rows'). A
-- line whose first mark is `#` is a note to the reader.
module CE.Lang.Table
  ( Blank (..)
  , Row (..)
  , rows
  ) where

import Data.Aeson
import qualified Data.Aeson.KeyMap as KM
import Data.Char (isSpace, toLower)
import Data.Proxy (Proxy (..))
import GHC.Generics
import GHC.TypeLits (KnownNat, Nat, natVal)

-- | The value a field holds when its table does not state it: the
-- empty list, the empty position, false, nothing — and, for a choice,
-- the variant the measuring side marks as its default.
class Blank a where
  blank :: a

class GBlank f where
  gblank :: f p

instance GBlank U1 where
  gblank = U1

instance (GBlank a, GBlank b) => GBlank (a :*: b) where
  gblank = gblank :*: gblank

instance (GBlank a) => GBlank (M1 i c a) where
  gblank = M1 gblank

instance (Blank a) => GBlank (K1 i a) where
  gblank = K1 blank

instance Blank [a] where
  blank = []

instance Blank Bool where
  blank = False

instance Blank Int where
  blank = 0

instance Blank (Maybe a) where
  blank = Nothing

instance (Blank a, Blank b) => Blank (a, b) where
  blank = (blank, blank)

instance (Blank a, Blank b, Blank c) => Blank (a, b, c) where
  blank = (blank, blank, blank)

-- | A record read and written by its field names — the Haskell field
-- less its @n@-letter prefix, in snake_case, the measuring side's own
-- spelling — every absent field its 'blank', an unknown key refused;
-- a choice by its constructor's name less the prefix, lower-cased
-- (`ModeReadWrite` is `readwrite`). The shapes derive their JSON and
-- their blank through it.
newtype Row (n :: Nat) a = Row a

rowOptions :: Integer -> Options
rowOptions n =
  defaultOptions
    { fieldLabelModifier = camelTo2 '_' . drop (fromInteger n)
    , constructorTagModifier = map toLower . drop (fromInteger n)
    , rejectUnknownFields = True
    }

instance (Generic a, GBlank (Rep a)) => Blank (Row n a) where
  blank = Row (to gblank)

instance (KnownNat n, Generic a, GToJSON' Value Zero (Rep a)) => ToJSON (Row n a) where
  toJSON (Row a) = genericToJSON (rowOptions (natVal (Proxy :: Proxy n))) a

instance
  (KnownNat n, Generic a, GFromJSON Zero (Rep a), GToJSON' Value Zero (Rep a), Blank a)
  => FromJSON (Row n a)
  where
  parseJSON v = Row <$> genericParseJSON o (over (genericToJSON o (blank :: a)) v)
   where
    o = rowOptions (natVal (Proxy :: Proxy n))

-- | The stated keys laid over the base's (left-biased).
over :: Value -> Value -> Value
over (Object base) (Object stated) = Object (KM.union stated base)
over _ stated = stated

notes :: String -> [String]
notes = filter (not . note) . lines
 where
  note l = take 1 (dropWhile isSpace l) == "#"

-- | A row table: a line opens a row — its first word the head, the
-- rest its words — and an indented line carries the row on.
rows :: String -> [(String, [String])]
rows = go . filter (not . all isSpace) . notes
 where
  go (l : rest)
    | (h : first) <- words l =
        let (more, next) = span indented rest
         in (h, first <> concatMap words more) : go next
  go _ = []
  indented (c : _) = isSpace c
  indented [] = False
