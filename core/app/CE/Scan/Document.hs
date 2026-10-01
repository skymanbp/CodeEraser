-- aeson 2.x object keys are Key, not String; string literals for
-- (.:)/(.=) need OverloadedStrings (Key's IsString instance).
{-# LANGUAGE OverloadedStrings #-}

-- | The `ce scan` document (plan v2.32 step 5; design booklet
-- docs/reference/authority-track.md §5), transcribed from the face that
-- assembled it on the measuring side (cli/src/scan/report.rs `Report`,
-- `findings_from`, `summarize`; cli/src/scan/wire.rs `judge`'s
-- `failed`): every measured file with its functions, the findings the
-- core's levels name — each with the line it crossed, the class
-- override's where the row's class overrides that code, else the echoed
-- grade table's — the four-number summary and the held conditions in
-- their canonical order. The measuring side sends its files and
-- functions as integers (the name-conformance bit and the three
-- complexity numbers it already read off the scan reply), the reply's
-- non-zero levels with each row's class, the echoed tables and the held
-- conditions by code; paths and function names are references.
module CE.Scan.Document (doc, findings) where

import CE.Document.Contract
import CE.Scan (conditionNames)
import CE.Scan.Cost (ruleNames)
import Data.Aeson (Value, object, (.=))
import Data.Foldable (asum)
import Data.List (nub, sort)
import qualified Data.Map.Strict as M

doc :: DocFamily
doc = (docFamily "scan" "ce.scan-report/0.2.0" statement checked assemble ["rules" .= ruleNames, "conditions" .= conditionNames]) {dfPretty = True}

-- | A file row is [f, total lines, comment lines, language code]; a
-- function row [u, f, start, end, lines, params, name conforms (0 /
-- 1), cyclomatic, cognitive, nesting], in file order; `rows` the wire
-- rows (a file's own row, then six per function); a level row [row,
-- level 1 warn / 2 fail, the row's class] for every non-zero level, in
-- row order; `grades` the echoed grade table [code, warn, fail], one
-- row per code; `overrides` the class lines [class, code, warn, fail];
-- `failed` the held conditions by code.
statement :: String
statement =
  "range files\nrange fns\nrange rows\n\
  \rows files 4 judged files - - -\nrows fns 10 judged fns files - - - - - - - -\n\
  \rows levels 3 judged rows - -\nrows grades 3 judged - - -\nrows overrides 4 judged - - - -\n\
  \rows failed 1 judged -\n\
  \ref path files\nref fn fns\n"

checked :: DocReq -> Maybe String
checked req =
  asum
    [ dense req "files" (range req "files")
    , dense req "fns" (range req "fns")
    , if range req "rows" == range req "files" + 6 * range req "fns" then Nothing else Just "ranges: rows is not files + 6 x fns"
    , if ascending (map (take 1 . drop 1) (rows req "fns")) then Nothing else Just "fns: not in file order"
    , asum [Just ("files " <> show i <> ": no language " <> show c) | (i, [_, _, _, c]) <- zip [0 :: Int ..] (rows req "files"), null (langName c)]
    , codes req "fns" 6 0 1
    , if strictly (map (take 1) (rows req "levels")) then Nothing else Just "levels: rows not strictly ascending"
    , codes req "levels" 1 1 2
    , dense req "grades" (toInteger (length (rows req "grades")))
    , codes req "grades" 0 0 6
    , codes req "overrides" 1 0 6
    , codes req "failed" 0 0 (toInteger (length conditionNames) - 1)
    , asum [Just ("levels " <> show r <> ": no grade for code " <> show c) | (r, c, _) <- layoutOf req, c >= toInteger (length (rows req "grades"))]
    ]
 where
  ascending xs = and (zipWith (<=) xs (drop 1 xs))
  strictly xs = and (zipWith (<) xs (drop 1 xs))

-- | The levelled rows: (row, code, the subject — its file and, on a
-- function's row, the function — with its line and value), merged in
-- row order out of the layout `rows_of` writes: a file's own row, then
-- six per function.
type Placed = (Integer, Integer, ((Integer, Maybe Integer), Integer, Integer))

layoutOf :: DocReq -> [Placed]
layoutOf req = merge (zip [0 ..] layout) (rows req "levels")
 where
  fns = M.fromListWith (flip (<>)) [(f, [r]) | r@(_ : f : _) <- rows req "fns"]
  layout = concat [fileRow r : concatMap fnRows (M.findWithDefault [] f fns) | r@(f : _) <- rows req "files"]
  fileRow r = case r of
    [f, total, _, _] -> (0, ((f, Nothing), 1, total))
    _ -> (0, ((-1, Nothing), 0, 0))
  fnRows r = case r of
    [u, f, start, _, ls, ps, ok, cc, coc, nest] -> zip [1 ..] [((f, Just u), start, v) | v <- [ls, ps, cc, coc, nest, 1 - ok]]
    _ -> []
  merge ((i, (c, s)) : rest) ls@((r : _) : more)
    | i == r = (i, c, s) : merge rest more
    | i < r = merge rest ls
    | otherwise = merge ((i, (c, s)) : rest) more
  merge _ _ = []

-- | The findings: each levelled row with its rule, level, value, the
-- line it crossed and its subject.
findings :: DocReq -> [Value]
findings req = zipWith finding (layoutOf req) (rows req "levels")
 where
  grades = M.fromList [(c, (w, f)) | [c, w, f] <- rows req "grades"]
  over = M.fromListWith (\_ first -> first) [((k, c), (w, f)) | [k, c, w, f] <- rows req "overrides"]
  finding (_, code, ((f, u), line, value)) lv = case lv of
    [_, level, cls] ->
      let (warn, failLine) = M.findWithDefault (M.findWithDefault (0, 0) code grades) (cls, code) over
       in object
            [ "file" .= ref "path" [f]
            , "line" .= line
            , "rule" .= spelled ruleNames code
            , "level" .= (if level == 2 then "fail" else "warn" :: String)
            , "value" .= value
            , "threshold" .= (if level == 2 then failLine else warn)
            , "subject" .= maybe (ref "path" [f]) (ref "fn" . pure) u
            ]
    _ -> object []

assemble :: DocReq -> Value
assemble req =
  object
    [ "schema" .= dfSchema doc
    , "files" .= map file (rows req "files")
    , "findings" .= hits
    , "summary" .= counted (words "files functions warns fails") [range req "files", range req "fns", count 1, count 2]
    , "failed" .= map (spelled conditionNames) (sort (nub (concat (rows req "failed"))))
    ]
 where
  hits = findings req
  count l = toInteger (length [() | [_, l', _] <- rows req "levels", l' == l])
  fns = M.fromListWith (flip (<>)) [(f, [r]) | r@(_ : f : _) <- rows req "fns"]
  file r = case r of
    [f, total, comments, lang] ->
      object
        [ "path" .= ref "path" [f]
        , "lang" .= langName lang
        , "total_lines" .= total
        , "comment_lines" .= comments
        , "functions" .= map function (M.findWithDefault [] f fns)
        ]
    _ -> object []
  function r = case r of
    [u, _, start, end, ls, ps, ok, cc, coc, nest] ->
      object
        [ "name" .= ref "fn" [u]
        , "start_line" .= start
        , "end_line" .= end
        , "lines" .= ls
        , "params" .= ps
        , "cyclomatic" .= cc
        , "cognitive" .= coc
        , "max_nesting" .= nest
        , "name_ok" .= (ok == 1)
        ]
    _ -> object []
