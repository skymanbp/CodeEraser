library(dplyr)
require(ggplot2)
source("helper.R")
requireNamespace("jsonlite")
suppressPackageStartupMessages(library(stats))
# comment
#' roxygen doc
#' @param x thing
helper <- function(a, b) { a && b || !a }
f = function(x) x
g <<- function(y) y
function(w) w -> h
lam <- \(z) z
`%+%` <- function(a, b) paste(a, b)
.hidden <- function() 1
print.myclass <- function(x, ...) cat("x")
setGeneric("area", function(shape) standardGeneric("area"))
setMethod("area", "Circle", function(shape) pi)
main <- function(n) {
  if (n > 1) print("x") else if (n == 0) print("y") else print("z")
  for (i in 1:3) { if (i == 1) next; if (i == 2) break }
  while (n > 0) n <- n - 1
  repeat { n <- n + 1; if (n > 3) break }
  b <- n > 0 & n < 5 | n == 9
  r <- tryCatch(risky(), error = function(e) NULL, finally = cleanup())
  s <- switch(n, "a" = 1, "b" = 2, 3)
  v <- ifelse(n > 0, 1, 0)
  x <- 'sq'; y <- "dq"; z <- r"(raw)"; k <- 1L; fl <- 2.5; cx <- 3i
  obj$method(1); pkg::fun(2); pkg:::hidden(3); helper(n, 1); ns@slot
  inner <- function(q) q
  helper(1)(2)
}
