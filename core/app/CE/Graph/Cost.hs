-- | Graph judgment constants — nothing but constants, so the plan
-- §7.4 sensitivity test and the ablation table have exactly one
-- target and a dead knob cannot hide (the CE.FourClass.Cost
-- convention). M5-2g adds minRung / entryMask / sccFloor here; M5-2a
-- ships only the size caps.
module CE.Graph.Cost (nodeCap, edgeCap) where

-- | Real oversize protection for graph requests (2026-08-12 decision:
-- the envelope byte precheck is relaxed for the trusted same-machine
-- child, so these caps are the guard). Sizing anchor: 100k LOC of the
-- M5-2 design brief measures ~20k nodes / ~60k edges, so the caps
-- carry ~6x headroom; a request at cap is ~8 MB, well under the
-- relaxed envelope ceiling. Over cap => degraded graph_too_large,
-- never a truncated graph.
nodeCap :: Int
nodeCap = 131072

edgeCap :: Int
edgeCap = 524288
