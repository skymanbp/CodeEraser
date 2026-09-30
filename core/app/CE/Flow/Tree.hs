-- | The flow request's four tables assembled into units (design
-- booklet §5.1): each unit's statements as a tree keyed by seq, its
-- variables by id, its accesses by statement in evaluation order.
-- Assembly IS the second half of the boundary contract — the rows
-- were shaped one by one in CE.Flow.Contract, and here the checks
-- that need the neighbours run: a statement's unit exists and its
-- seq continues the unit's pre-order, its parent came before it, a
-- variable's declaring statement is in its unit, an access names a
-- statement and a variable that exist, and the unit's declared
-- parameter count is the count the variable table shows. The tree's
-- SHAPE (which kinds may sit under which) is CE.Flow.Shape's.
module CE.Flow.Tree (Stmt (..), Unit (..), Use (..), Var (..), ancestors, build, hasFlag, kidsOf) where

import CE.Flow.Cost (varParam)
import Control.Monad (foldM)
import Data.Bits (testBit)
import qualified Data.IntMap.Strict as IM

-- | One statement: its row index in the request (the name a refusal
-- calls it by), its seq, its parent's seq (−1 = the unit body), its
-- kind, its flag bits and its aux (a jump's target seq, else 0).
data Stmt = Stmt {sIdx :: Int, sSeq :: Int, sParent :: Int, sKind :: Int, sFlags :: Int, sAux :: Int}

-- | One variable: row index, id, declaring seq (−1 = a parameter),
-- flag bits.
data Var = Var {vIdx :: Int, vId :: Int, vDecl :: Int, vFlags :: Int}

-- | One access: the statement, the variable, the mode.
data Use = Use {uSeq :: Int, uVar :: Int, uMode :: Int}

-- | One unit and everything the tables said about it.
data Unit = Unit
  { unitId :: Int
  , unitIdx :: Int
  , unitLang :: Int
  , unitParams :: Int
  , stmts :: IM.IntMap Stmt
  -- ^ by seq
  , kids :: IM.IntMap [Int]
  -- ^ parent seq (−1 = the unit body) → children, ascending
  , vars :: IM.IntMap Var
  -- ^ by id
  , uses :: IM.IntMap [Use]
  -- ^ by seq, evaluation order
  }

-- | The children of a statement (or of the unit body at −1).
kidsOf :: Unit -> Int -> [Int]
kidsOf u p = IM.findWithDefault [] p (kids u)

hasFlag :: Int -> Stmt -> Bool
hasFlag bit s = testBit (sFlags s) bit

-- | The proper ancestors of a statement, innermost first.
ancestors :: Unit -> Int -> [Int]
ancestors u s = case IM.lookup s (stmts u) of
  Just Stmt {sParent = p} | p >= 0 -> p : ancestors u p
  _ -> []

-- | The units, or the first neighbour-level offence in request
-- order: statements, then variables, then accesses, then the
-- parameter counts. Every row arrives already shaped (width and
-- ranges), so the patterns below are total on what reaches them.
build :: [[Integer]] -> [[Integer]] -> [[Integer]] -> [[Integer]] -> Either String (IM.IntMap Unit)
build us ss vs xs = do
  let empty = IM.fromList [(fromInteger u, Unit (fromInteger u) i (fromInteger l) (fromInteger p) IM.empty IM.empty IM.empty IM.empty) | (i, [u, l, p]) <- zip [0 ..] us]
  withStmts <- grouped "stmt" stmtRow empty ss
  withVars <- grouped "var" varRow withStmts vs
  withUses <- grouped "use" useRow withVars xs
  mapM_ paramsAgree (IM.elems withUses)
  pure (IM.map (\u -> u {kids = childMap (stmts u)}) withUses)

-- | One table's rows folded into their units in request order. The
-- driver owns what every table shares — the row's unit must be
-- declared, and a refusal names the table and the row — and threads
-- the (unit, key) pair the previous row settled at, so each table's
-- step can ask whether this row continues it.
grouped :: String -> ((Int, Int) -> Int -> [Integer] -> Unit -> Either String (Unit, Int)) -> IM.IntMap Unit -> [[Integer]] -> Either String (IM.IntMap Unit)
grouped what step units rows = fst <$> foldM go (units, (-1, -1)) (zip [0 ..] rows)
 where
  go (m, prev) (i, row@(u' : _)) = case IM.lookup u m of
    Nothing -> Left (at i "unit not declared")
    Just un -> either (Left . at i) (\(un', key) -> Right (IM.insert u un' m, (u, key))) (step prev i row un)
   where
    u = fromInteger u'
  go _ (i, []) = Left (at i ("malformed " <> what))
  at i msg = what <> " " <> show i <> ": " <> msg

-- | The key a row must carry to continue the previous row's unit: the
-- previous key + 1, or 0 when it opens a new unit.
continues :: (Int, Int) -> Int -> Int
continues (pu, pk) u = if u == pu then pk + 1 else 0

-- | A statement: units ascending, seq continuing the unit's
-- pre-order, the parent earlier than the statement.
stmtRow :: (Int, Int) -> Int -> [Integer] -> Unit -> Either String (Unit, Int)
stmtRow prev i [u', seq', parent', kind', flags', aux'] un
  | u < fst prev = Left "not strictly ascending"
  | continues prev u /= sq = Left "seq not consecutive"
  | par >= sq = Left "parent not before"
  | otherwise = Right (un {stmts = IM.insert sq (Stmt i sq par (fromInteger kind') (fromInteger flags') (fromInteger aux')) (stmts un)}, sq)
 where
  u = fromInteger u'
  sq = fromInteger seq'
  par = fromInteger parent'
stmtRow _ _ _ _ = Left "malformed stmt"

-- | A variable: units ascending, ids continuing the unit's numbering,
-- the declaring statement in the unit, and a parameter exactly a
-- variable declared at no statement.
varRow :: (Int, Int) -> Int -> [Integer] -> Unit -> Either String (Unit, Int)
varRow prev i [u', v', decl', flags'] un
  | u < fst prev = Left "not strictly ascending"
  | continues prev u /= v = Left "v not consecutive"
  | decl >= IM.size (stmts un) = Left "declSeq outside the unit"
  | param && decl /= -1 = Left "param declared at a statement"
  | not param && decl == -1 = Left "local declared at no statement"
  | otherwise = Right (un {vars = IM.insert v var (vars un)}, v)
 where
  u = fromInteger u'
  v = fromInteger v'
  decl = fromInteger decl'
  var = Var i v decl (fromInteger flags')
  param = testBit (vFlags var) varParam
varRow _ _ _ _ = Left "malformed var"

-- | An access: the table ordered by (unit, seq) — several accesses of
-- one statement stay in evaluation order and may repeat — naming a
-- statement and a variable that exist.
useRow :: (Int, Int) -> Int -> [Integer] -> Unit -> Either String (Unit, Int)
useRow prev _ [u', seq', v', mode'] un
  | (u, sq) < prev = Left "out of order"
  | sq >= IM.size (stmts un) = Left "seq outside the unit"
  | v >= IM.size (vars un) = Left "var not declared"
  | otherwise = Right (un {uses = IM.insertWith (\new old -> old ++ new) sq [Use sq v (fromInteger mode')] (uses un)}, sq)
 where
  u = fromInteger u'
  sq = fromInteger seq'
  v = fromInteger v'
useRow _ _ _ _ = Left "malformed use"

-- | Parent → children ascending (the elems walk is ascending by seq).
childMap :: IM.IntMap Stmt -> IM.IntMap [Int]
childMap m = IM.fromListWith (\new old -> old ++ new) [(sParent s, [sSeq s]) | s <- IM.elems m]

-- | The unit row's parameter count is the variable table's.
paramsAgree :: Unit -> Either String ()
paramsAgree u
  | length [() | v <- IM.elems (vars u), testBit (vFlags v) varParam] == unitParams u = Right ()
  | otherwise = Left ("unit " <> show (unitIdx u) <> ": params disagree with the var table")
