-- | The small trees the clone batteries name their cases with (plan v2.35:
-- the sequence bound's battery, CloneSeqProps, and the banded distance's,
-- CloneWithinProps), written once in the wire's (lab, lld) form.
module CloneNamedTrees (one, other, fork, relabelled, stick) where

-- | Two single nodes, labels 0 and 1: distance 1, a relabel.
one, other :: ([Int], [Int])
one = ([0], [0])
other = ([1], [0])

-- | A root over two leaves, all labelled 0; the same shape with the right
-- leaf relabelled (distance 1); a two-node chain (fork − one leaf, distance 1).
fork, relabelled, stick :: ([Int], [Int])
fork = ([0, 0, 0], [0, 1, 0])
relabelled = ([0, 1, 0], [0, 1, 0])
stick = ([0, 0], [0, 0])
