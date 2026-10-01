{-# LANGUAGE LambdaCase #-}
-- | The Haskell slot-table probe (plan v2.31 step 7): every position
-- class the merge family reads, in one module.
module Probe (Shape (..), Tally (..), area, tally, run) where

import qualified Data.Map.Strict as M
import Data.List (foldl')

-- | A sum type and a record.
data Shape = Circle Double | Rect Double Double
  deriving (Show, Eq)

data Tally = Tally {tCount :: Int, tNames :: [String]}

newtype Wrap a = Wrap a

type Table = M.Map String Int

class Sized a where
  size :: a -> Int
  size _ = 0

instance Sized Shape where
  size (Circle _) = 1
  size Rect {} = 2

infixl 6 |+|

(|+|) :: Int -> Int -> Int
a |+| b = a + b

-- | Guards, where, case and a lambda.
area :: Shape -> Double
area s
  | Circle r <- s = pi * r * r
  | otherwise = case s of
      Rect w h -> w * h
      _ -> scale 0
 where
  scale k = k * 2

tally :: [String] -> Tally
tally names = Tally {tCount = length names, tNames = map (\n -> n ++ "!") names}

-- | A do block with binds, lets, a conditional and a section.
run :: Table -> IO Int
run table = do
  let total = foldl' (+) 0 (M.elems table)
      bumped = (+ 1) total
  line <- getLine
  if null line
    then pure bumped
    else do
      print [x * 2 | x <- [1 .. total], even x]
      pure (negate total)

pick :: Maybe Int -> Int
pick = \case
  Just n -> n `div` 2
  Nothing -> let z = 3 in z

update :: Tally -> Tally
update t = t {tCount = tCount t + 1}

-- a plain comment, a tuple, a char, a float and two sections
pair :: (Char, Double)
pair = ('a', 1.5)

halve :: [Int] -> [Int]
halve = map (2 *) . filter (> 0)
