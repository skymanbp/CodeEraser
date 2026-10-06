-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The bags.request contract (plan v2.33 W6): the request record, its
-- one cap and the first offence by name. The handler is CE.Similar.Bags.
module CE.Similar.Bags.Contract (BagsReq (..), Unit, bagsCap, overCap, offence) where

import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.List (findIndex)

-- | key, kind word, return flag, callees, literal features, structure
-- histogram, doc lines.
type Unit = (String, String, Maybe Integer, [String], [String], [[Integer]], [String])

data BagsReq = BagsReq
  { reqId :: Value
  , units :: [Unit]
  , texts :: [String]
  , inspect :: [Integer]
  }

instance FromJSON BagsReq where
  parseJSON = withObject "BagsReq" $ \o ->
    BagsReq
      <$> o .: "id"
      <*> o .:? "units" .!= []
      <*> o .:? "texts" .!= []
      <*> o .:? "inspect" .!= []

-- | Rows, texts, code points and every list element of every unit count
-- against one cap (review C15's stance — every request dimension counts).
bagsCap :: Int
bagsCap = 1048576

overCap :: BagsReq -> Bool
overCap = (> bagsCap) . itemsOf

itemsOf :: BagsReq -> Int
itemsOf r = length (units r) + length (texts r) + length (inspect r) + sum (map size (units r))
 where
  size (_, _, _, cs, ls, ss, ds) = length cs + length ls + length ss + length ds

-- | The first unit whose return flag, structure rows or order break the
-- request shape (CE.Similar.Bags), else the first code point out of range.
offence :: BagsReq -> Maybe String
offence r = case findIndex bad (units r) of
  Just i -> Just ("units " <> show i <> ": need ret null/0/1 and [[kind,n]] strictly ascending, 0 <= kind < 2^64, 1 <= n < 2^32")
  Nothing
    | any (\c -> c < 0 || c > 0x10FFFF) (inspect r) -> Just "inspect: a code point out of 0..0x10FFFF"
    | otherwise -> Nothing
 where
  bad (_, _, ret, _, _, ss, _) = maybe False (`notElem` [0, 1]) ret || not (ascending ss)
  ascending ss = all row ss && and (zipWith (<) kinds (drop 1 kinds)) where kinds = [k | (k : _) <- ss]
  row [k, n] = k >= 0 && k < 2 ^ (64 :: Int) && n >= 1 && n < 2 ^ (32 :: Int)
  row _ = False
