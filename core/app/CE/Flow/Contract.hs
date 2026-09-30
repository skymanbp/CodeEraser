-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The flow.request shape and its boundary contract (design booklet
-- §3's refusal rule and §5.1's tables): what a well-formed request IS
-- — four integer tables, each row the right width with the right
-- ranges, the units strictly ascending — and which dimension the cap
-- prices. The neighbour-level half of the contract (a statement's
-- unit and parent, a variable's declaring statement, an access's
-- targets, the tree's shape) runs in CE.Flow.Tree and CE.Flow.Shape
-- once the rows are shaped; the units they assemble are the same
-- thunk the judgment reads, so nothing is checked or built twice.
module CE.Flow.Contract (FlowReq (..), offence, overCap) where

import CE.Flow.Cost
import qualified CE.Flow.Shape as Shape
import CE.Flow.Tree (Unit, build)
import CE.Wire (ascendingOn, rowCheck)
import Control.Applicative ((<|>))
import Data.Aeson (FromJSON (..), Value, withObject, (.!=), (.:), (.:?))
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

-- | The request: id and the four tables, each absent read as empty.
-- `unitsOf` is built lazily from the tables and shared by the
-- contract and the judgment.
data FlowReq = FlowReq
  { reqId :: Value
  , unitRows :: [[Integer]]
  , stmtRows :: [[Integer]]
  , varRows :: [[Integer]]
  , useRows :: [[Integer]]
  , unitsOf :: Either String (IM.IntMap Unit)
  }

instance FromJSON FlowReq where
  parseJSON = withObject "FlowReq" $ \o -> do
    i <- o .: "id"
    us <- o .:? "units" .!= []
    ss <- o .:? "stmts" .!= []
    vs <- o .:? "vars" .!= []
    xs <- o .:? "uses" .!= []
    pure (FlowReq i us ss vs xs (build us ss vs xs))

-- | The four tables are one dimension against one cap.
overCap :: FlowReq -> Bool
overCap req = toInteger (sum (map length [unitRows req, stmtRows req, varRows req, useRows req])) > rowCap

-- | The first offender in request order: every row's shape table by
-- table, the unit order, then the neighbour-level checks and the
-- tree's shape.
offence :: FlowReq -> Maybe String
offence req =
  asum
    [ asum (zipWith unitShape [0 ..] (unitRows req))
    , ascendingOn "unit" (take 1) (unitRows req)
    , asum (zipWith stmtShape [0 ..] (stmtRows req))
    , asum (zipWith varShape [0 ..] (varRows req))
    , asum (zipWith useShape [0 ..] (useRows req))
    ]
    <|> either Just Shape.offence (unitsOf req)

unitShape :: Int -> [Integer] -> Maybe String
unitShape = rowCheck "unit" "malformed unit (need [u,lang,params])" 3 (negative "negative unit value" 0)

-- | [u, seq, parent, kind, flags, aux]: parent may be −1 (the unit
-- body), everything else is non-negative; kind and flags are bounded.
stmtShape :: Int -> [Integer] -> Maybe String
stmtShape = rowCheck "stmt" "malformed stmt (need [u,seq,parent,kind,flags,aux])" 6 checks
 where
  checks row = case row of
    [u, sq, parent, kind, flags, aux]
      | any (< 0) [u, sq, aux] || parent < -1 || kind < 0 || flags < 0 -> Just "negative stmt value"
      | kind > toInteger kindCeil -> Just "unknown kind"
      | flags > stmtFlagCeil -> Just "flags outside five bits"
    _ -> Nothing

-- | [u, v, declSeq, flags]: declSeq may be −1 (a parameter).
varShape :: Int -> [Integer] -> Maybe String
varShape = rowCheck "var" "malformed var (need [u,v,declSeq,flags])" 4 checks
 where
  checks row = case row of
    [u, v, decl, flags]
      | any (< 0) [u, v, flags] || decl < -1 -> Just "negative var value"
      | flags > varFlagCeil -> Just "flags outside four bits"
    _ -> Nothing

-- | [u, seq, v, mode]: all non-negative, mode bounded.
useShape :: Int -> [Integer] -> Maybe String
useShape = rowCheck "use" "malformed use (need [u,seq,v,mode])" 4 checks
 where
  checks row = case row of
    [_, _, _, mode] | mode > modeCeil -> Just "unknown mode"
    _ -> negative "negative use value" 0 row

-- | Every value at or above the floor.
negative :: String -> Integer -> [Integer] -> Maybe String
negative message floor' row
  | any (< floor') row = Just message
  | otherwise = Nothing
