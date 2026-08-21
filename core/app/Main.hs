{-# LANGUAGE LambdaCase #-}

-- | ce-core entry point: @--version@, or serve NDJSON on stdio until EOF.
-- Binary-mode ByteString I/O per ADR-003 (avoids the Windows code-page
-- trap, GHC #10762 / #15021).
module Main (main) where

import CE.Protocol (internalError, respond)
import Control.Exception (SomeAsyncException, SomeException, evaluate, fromException, throwIO, try)
import qualified Data.ByteString.Builder as BB
import qualified Data.ByteString.Char8 as B8
import qualified Data.ByteString.Lazy as BL
import Data.Version (showVersion)
import qualified Paths_ce_core as Meta
import System.Environment (getArgs)
import System.Exit (exitFailure)
import System.IO (BufferMode (LineBuffering), Handle, hSetBinaryMode, hSetBuffering, stderr, stdin, stdout)

-- one source — the .cabal version, via the cabal-generated module
-- (a second literal here is how a v0.2.0 asset self-reported 0.0.1)
coreVersion :: String
coreVersion = showVersion Meta.version

main :: IO ()
main = getArgs >>= \case
  ["--version"] -> putStrLn ("ce-core " <> coreVersion)
  []            -> serve
  args          -> do
    hPutUtf8 stderr ("ce-core: unknown arguments: " <> unwords args)
    exitFailure

-- | The one diagnostic that can carry ARBITRARY text (argv), and it
-- runs before serve's hSetBinaryMode: hPutStrLn would encode through
-- the handle's locale codec, so a non-ASCII argument on a legacy
-- Windows code page died in commitBuffer with the diagnostic never
-- printed (2026-08-20 protocol review) — the exact trap ADR-003 keeps
-- the wire clear of. Same road as every reply: bytes, UTF-8.
-- (@--version@ stays on putStrLn: showVersion is digits and dots, and
-- its text-mode newline is what the shipped output already is.)
hPutUtf8 :: Handle -> String -> IO ()
hPutUtf8 h = B8.hPutStrLn h . BL.toStrict . BB.toLazyByteString . BB.stringUtf8

serve :: IO ()
serve = do
  hSetBinaryMode stdin True
  hSetBinaryMode stdout True
  hSetBuffering stdout LineBuffering
  loop B8.empty
 where
  loop residue =
    frame stdin residue >>= \case
      Done -> pure ()
      Line line rest -> answer line >> loop rest
      -- the over-budget prefix is still what respond judges — the
      -- ceiling belongs to the envelope layer, not to this reader —
      -- and the runaway tail is swallowed, never buffered
      Overlong prefix -> answer prefix >> drainLine stdin >>= loop

-- | One request line in, one response line out. "never a crash" as
-- STRUCTURE, not argument (M5-close review LOW): respond is pure and
-- its strict ByteString is fully materialized by evaluate, so any
-- defect surfaces here as an error line and the session survives.
answer :: B8.ByteString -> IO ()
answer line = do
  reply <- try (evaluate (respond coreVersion line))
  out <- either onCrash pure reply
  B8.hPutStrLn stdout out

-- an ASYNC exception (interrupt, RTS kill) is not a request defect
-- — rethrow so the barrier never makes the process uninterruptible
-- (clearance review on the first cut's SomeException net)
onCrash :: SomeException -> IO B8.ByteString
onCrash e = case fromException e :: Maybe SomeAsyncException of
  Just _ -> throwIO e
  Nothing -> pure (internalError (show e))

-- | What one framing read yielded: a line (newline stripped, residue
-- kept for the next frame), a line whose length passed the read
-- budget before any newline arrived, or clean EOF.
data Frame
  = Done
  | Line B8.ByteString B8.ByteString
  | Overlong B8.ByteString

-- | A MEMORY bound, not the protocol ceiling: CE.Protocol.maxLineBytes
-- (32 MiB) stays the single authority on what is in contract, which is
-- why an over-budget prefix is handed to respond for the verdict
-- rather than answered here. This number must only stay ABOVE that
-- ceiling so no in-contract line is ever cut short. Until it existed,
-- B8.hGetLine materialized the whole line BEFORE the guard could look
-- at it — the advertised 32 MiB ceiling was a report, not a bound, and
-- one newline-free flood grew the heap until the process died
-- (2026-08-20 protocol review).
readBudget :: Int
readBudget = 67108864

chunkBytes :: Int
chunkBytes = 65536

-- | Read incrementally up to the budget, splitting at the first '\n'.
-- Only '\n' is stripped: a CRLF client's '\r' stays inside the line
-- exactly as B8.hGetLine left it (JSON whitespace — the goldens and
-- the daemon's framing must not shift by a byte).
frame :: Handle -> B8.ByteString -> IO Frame
frame h = go [] 0
 where
  go acc len piece =
    let acc' = piece : acc
        len' = len + B8.length piece
     in case B8.elemIndex '\n' piece of
          Just i -> pure (Line (whole (B8.take i piece : acc)) (B8.drop (i + 1) piece))
          Nothing
            | len' > readBudget -> pure (Overlong (whole acc'))
            | otherwise -> do
                chunk <- B8.hGetSome h chunkBytes
                if B8.null chunk
                  then pure (if len' == 0 then Done else Line (whole acc') B8.empty)
                  else go acc' len' chunk
  whole = B8.concat . reverse

-- | Swallow the tail of an over-budget line in constant memory, so
-- the remainder is never read as further requests (and never held).
-- Bytes past its newline are the next frame's head.
drainLine :: Handle -> IO B8.ByteString
drainLine h = do
  chunk <- B8.hGetSome h chunkBytes
  if B8.null chunk
    then pure B8.empty
    else case B8.elemIndex '\n' chunk of
      Just i -> pure (B8.drop (i + 1) chunk)
      Nothing -> drainLine h
