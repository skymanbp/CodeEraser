{-# LANGUAGE LambdaCase #-}

-- | ce-core entry point: @--version@, or serve NDJSON on stdio until EOF.
-- Binary-mode ByteString I/O per ADR-003 (avoids the Windows code-page
-- trap, GHC #10762 / #15021).
module Main (main) where

import CE.Protocol (internalError, respond)
import Control.Exception (SomeException, evaluate, try)
import qualified Data.ByteString.Char8 as B8
import System.Environment (getArgs)
import System.Exit (exitFailure)
import System.IO

coreVersion :: String
coreVersion = "0.0.1"

main :: IO ()
main = getArgs >>= \case
  ["--version"] -> putStrLn ("ce-core " <> coreVersion)
  []            -> serve
  args          -> do
    hPutStrLn stderr ("ce-core: unknown arguments: " <> unwords args)
    exitFailure

serve :: IO ()
serve = do
  hSetBinaryMode stdin True
  hSetBinaryMode stdout True
  hSetBuffering stdout LineBuffering
  loop
 where
  loop = do
    end <- hIsEOF stdin
    if end
      then pure ()
      else do
        line <- B8.hGetLine stdin
        -- "never a crash" as STRUCTURE, not argument (M5-close review
        -- LOW): respond is pure and its strict ByteString is fully
        -- materialized by evaluate, so any defect surfaces here as an
        -- error line and the session survives
        reply <- try (evaluate (respond coreVersion line))
        B8.hPutStrLn stdout (either onCrash id reply)
        loop
  onCrash :: SomeException -> B8.ByteString
  onCrash = internalError . show
