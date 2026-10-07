-- | The whitepaper's worked examples (contracts/fixtures/scan/
-- whitepaper.ndjson) written as structure trees for ReferenceScan:
-- one tree per page function, keyed by the register line's name and
-- language, so the reference reads the PAGE's structure while the
-- core reads the events the Rust emitter wrote for each port. Where
-- a port changes the structure the tree says so: Go's `default:` is
-- no decision while the C family's and Java's is (register D2), and
-- the Lua and R ports of toRegexp spell the ternary as an if.
module ScanPageTrees (pageTree) where

import ReferenceScanGen

pageTree :: String -> String -> Maybe [Stmt]
pageTree name ext = lookup name table
 where
  table =
    [ ("sumOfPrimes", [loop Leaf [loop Leaf [If Leaf [Jump True] NoElse]]])
    , ("getWords", [Switch Leaf [Case True [], Case True [], Case True [], Case (ext /= "go") []]])
    , ("negated", [If (Bin And Leaf (Paren (Bin And Leaf Leaf))) [] NoElse])
    , ("sequences", [If sequences [] NoElse])
    , ("my_method", tryCatch)
    , ("myMethod", tryCatch)
    , ("myMethod2", [Do (Lambda [If Leaf [] NoElse])])
    , ("overriddenSymbolFrom", overridden)
    , ("addVersion", addVersion)
    , ("toRegexp", (if ext `elem` ["lua", "R"] then If either' [] NoElse else Do (Ternary either' Leaf Leaf)) : regexpLoop)
    ]
  either' = Bin Or Leaf Leaf

-- | `a && b && c || d || e && f`, operators left to right.
sequences :: Expr
sequences = Bin Or (Bin Or (Bin And (Bin And Leaf Leaf) Leaf) Leaf) (Bin And Leaf Leaf)

both :: Expr
both = Bin And Leaf Leaf

-- | A loop without Python's `else`.
loop :: Expr -> [Stmt] -> Stmt
loop h b = Loop h b Nothing

tryCatch :: [Stmt]
tryCatch = [Try [If Leaf [loop Leaf [loop Leaf []]] NoElse] [[If Leaf [] NoElse]] []]

overridden :: [Stmt]
overridden =
  [ If Leaf [] NoElse
  , loop Leaf [If both [If Leaf [If Leaf [If Leaf [] NoElse] (ElseIf Leaf [] NoElse)] NoElse] NoElse]
  , If Leaf [] NoElse
  ]

-- | while (true) { try { synchronized { … } } catch … catch … }.
addVersion :: [Stmt]
addVersion =
  [ loop
      Leaf
      [ Try
          [Block [If Leaf [If Leaf [] NoElse, If Leaf [loop Leaf [If Leaf [] NoElse, If both [] NoElse]] NoElse] NoElse]]
          [[Try [If both [] NoElse] [[]] []], []]
          []
      ]
  ]

regexpLoop :: [Stmt]
regexpLoop =
  [ loop
      Leaf
      [ If
          Leaf
          []
          ( ElseIf
              Leaf
              [If both [If both [] (Else True [])] (Else True [])]
              (ElseIf Leaf [] (ElseIf Leaf [] (Else True [])))
          )
      ]
  ]
