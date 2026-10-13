-- | Every judgment family's name and capability version, spelled once
-- (plan v2.33 step 11a, C28): the dispatch table (CE.Protocol) derives
-- the capability `<name>/<version>` and the request type
-- `<name>.request` from it, and each family's decode-error prefix and
-- reply type `<name>.result` read the same word. The names live in
-- this leaf module rather than in CE.Protocol's table itself because
-- the table imports every family module, so no family could read it.
module CE.Wire.Families
  ( FamilyName (..)
  , arch
  , audit
  , bags
  , candidates
  , clone
  , docdup
  , docpairs
  , document
  , erase
  , flow
  , fourclass
  , graph
  , merge
  , moves
  , query
  , rank
  , resolve
  , scan
  , similar
  , structure
  , tables
  , tombstone
  , trend
  , verdict
  ) where

-- | A family's word and its capability's version.
data FamilyName = FamilyName {nameWord :: String, nameVersion :: Int}

-- | A family at version 1, where every family but two stands.
first :: String -> FamilyName
first w = FamilyName w 1

arch, audit, bags, candidates, clone, docdup, docpairs, document, erase, flow, graph :: FamilyName
arch = first "arch"
audit = first "audit"
bags = first "bags"
candidates = first "candidates"
clone = first "clone"
docdup = first "docdup"
docpairs = first "docpairs"
document = first "document"
erase = first "erase"
flow = first "flow"
graph = first "graph"

merge, moves, query, rank, resolve, scan, similar, structure, tables, tombstone, verdict :: FamilyName
merge = first "merge"
moves = first "moves"
query = first "query"
rank = first "rank"
resolve = first "resolve"
scan = first "scan"
similar = first "similar"
structure = first "structure"
tables = first "tables"
tombstone = first "tombstone"
verdict = first "verdict"

-- | The two families whose capability moved to a second version.
fourclass, trend :: FamilyName
fourclass = FamilyName "fourclass" 2
trend = FamilyName "trend" 2
