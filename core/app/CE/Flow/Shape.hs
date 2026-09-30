-- | The statement tree's shape contract (design booklet §5.1 as
-- decided at step 3): which flags a kind may carry, which kinds may
-- carry children, where a case, a catch and a finally may sit, how
-- many branches an if has, and that every jump names a target the
-- control-flow graph can honour — a break or continue an enclosing
-- loop (a break also a switch), a goto a label of the unit. Each
-- refusal names the offending row, so the measuring side learns
-- which statement its FlowSpec table lowered wrongly.
module CE.Flow.Shape (offence) where

import CE.Flow.Cost
import CE.Flow.Tree (Stmt (..), Unit (..), ancestors, hasFlag, kidsOf)
import Data.Foldable (asum)
import qualified Data.IntMap.Strict as IM

-- | The first shape offence in request order (units ascending,
-- statements by seq — the order the rows arrived in).
offence :: IM.IntMap Unit -> Maybe String
offence units = asum [asum (map (stmtOffence u) (IM.elems (stmts u))) | u <- IM.elems units]

stmtOffence :: Unit -> Stmt -> Maybe String
stmtOffence u s =
  fmap (("stmt " <> show (sIdx s) <> ": ") <>) $
    asum
      [ flagsOnKind s
      , auxOnKind s
      , jumpTarget u s
      , branches u s
      , switchArms u s
      , seat u s
      , tryOrder u s
      , leaf u s
      ]

-- | else only on an if or a switch, infinite only on a loop,
-- fallthrough only on a case; dynamic and empty ride anywhere.
flagsOnKind :: Stmt -> Maybe String
flagsOnKind s
  | any misplaced [(flagElse, [kindIf, kindSwitch]), (flagInfinite, [kindLoop]), (flagFallthrough, [kindCase])] = Just "flag on the wrong kind"
  | otherwise = Nothing
 where
  misplaced (bit, kinds) = hasFlag bit s && sKind s `notElem` kinds

-- | Only a jump carries an aux; every other kind sends 0.
auxOnKind :: Stmt -> Maybe String
auxOnKind s
  | isJump s || sAux s == 0 = Nothing
  | otherwise = Just "aux on a non-jump"

isJump :: Stmt -> Bool
isJump s = sKind s `elem` [kindBreak, kindContinue, kindGoto]

-- | A break leaves an enclosing loop or switch, a continue restarts
-- an enclosing loop, a goto lands on a label of this unit.
jumpTarget :: Unit -> Stmt -> Maybe String
jumpTarget u s
  | sKind s == kindBreak && not (enclosing [kindLoop, kindSwitch]) = Just "break target is not an enclosing loop or switch"
  | sKind s == kindContinue && not (enclosing [kindLoop]) = Just "continue target is not an enclosing loop"
  | sKind s == kindGoto && kindAt u (sAux s) /= Just kindLabel = Just "goto target is not a label"
  | otherwise = Nothing
 where
  enclosing kinds = sAux s `elem` ancestors u (sSeq s) && maybe False (`elem` kinds) (kindAt u (sAux s))

kindAt :: Unit -> Int -> Maybe Int
kindAt u seq' = sKind <$> IM.lookup seq' (stmts u)

-- | An if runs one branch, or two under `flagElse`, and nothing else.
branches :: Unit -> Stmt -> Maybe String
branches u s
  | sKind s == kindIf && length (kidsOf u (sSeq s)) /= (if hasFlag flagElse s then 2 else 1) = Just "if branches disagree with has_else"
  | otherwise = Nothing

-- | A switch's children are cases, and a default arm needs one.
switchArms :: Unit -> Stmt -> Maybe String
switchArms u s
  | sKind s /= kindSwitch = Nothing
  | any (\k -> kindAt u k /= Just kindCase) arms = Just "switch child is not a case"
  | hasFlag flagElse s && null arms = Just "default on an empty switch"
  | otherwise = Nothing
 where
  arms = kidsOf u (sSeq s)

-- | A case sits under a switch, a catch or a finally under a try.
seat :: Unit -> Stmt -> Maybe String
seat u s
  | sKind s == kindCase && parentKind /= Just kindSwitch = Just "case outside a switch"
  | sKind s `elem` [kindCatch, kindFinally] && parentKind /= Just kindTry = Just "handler outside a try"
  | otherwise = Nothing
 where
  parentKind = kindAt u (sParent s)

-- | A try's children run body first, then its catches, then at most
-- one finally, last.
tryOrder :: Unit -> Stmt -> Maybe String
tryOrder u s
  | sKind s /= kindTry = Nothing
  | otherwise = asum (zipWith rule kinds (drop 1 kinds)) <> second
 where
  kinds = [k | c <- kidsOf u (sSeq s), Just k <- [kindAt u c]]
  handler k = k == kindCatch || k == kindFinally
  rule before after
    | handler before && not (handler after) = Just "body statement after a handler"
    | before == kindFinally && after == kindCatch = Just "catch after finally"
    | otherwise = Nothing
  second = if length (filter (== kindFinally) kinds) > 1 then Just "second finally" else Nothing

-- | A plain statement, an exit and a jump have no children.
leaf :: Unit -> Stmt -> Maybe String
leaf u s
  | sKind s `elem` [kindStmt, kindReturn, kindThrow, kindBreak, kindContinue, kindGoto, kindNoreturn] && not (null (kidsOf u (sSeq s))) = Just "children under a leaf statement"
  | otherwise = Nothing
