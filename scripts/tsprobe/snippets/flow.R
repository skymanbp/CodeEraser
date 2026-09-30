# Flow sample (analysis-track §5.3, round 3): every statement,
# declaration and access shape the flow/1 lowering table names, in R 4.x.

counter <- 0

branches <- function(a, b = 2, ...) {
  if (a > 0) {
    label <- "pos"
  } else if (a < 0) {
    label <- "neg"
  } else label <- "zero"
  total <- 0
  for (i in seq_len(b)) {
    if (i == 3) next
    if (i > 7) break
    total <- total + i
  }
  n <- 3
  while (n > 0) n <- n - 1
  while (TRUE) {
    if (total > 10) break
    total = total + 1
  }
  repeat {
    n <- n + 1
    if (n > 5) break
  }
  total -> saved
  counter <<- counter + total
  saved ->> last_total
  list(label, saved, ...)
}

accesses <- function(df, flag, items = list()) {
  x <- 0
  x <- x + 1
  df$col <- x
  df[["other"]] <- x
  items[1] <- x
  names(items) <- "first"
  y <- if (flag) x else 0
  flag && (z <- 1)
  flag || (w <- 2L)
  pick <- ifelse(flag, x, y)
  return(pick)
}

closures <- function(values) {
  base <- 10
  scale <- 2
  helper <- function(v) {
    scale <<- scale + 1
    v * base
  }
  add <- \(d) d + base
  sapply(values, function(v) v + scale)
  helper(values[[1]]) + add(1)
}

dynamic <- function(code) {
  value <- eval(parse(text = code))
  assign("hidden", value)
  get("hidden")
}

fatal <- function(code) {
  if (code > 0) {
    stop("fatal")
  }
  quit(status = 1)
  after <- 1
  after
}

checked <- function(x) {
  if (is.null(x)) return(NULL)
  rlang::abort("bad")
}
