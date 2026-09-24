x$y$z <- function(a) a
obj@slot <- 1
assign("dyn", function() 1)
f <- function(a, b = 2, ...) { g <- function() a; g() }
if (TRUE) { 1 } else { 2 }
for (i in seq_len(3)) next
lapply(1:3, function(i) i)
`my fun` <- function() 1
h = \(x) x + 1
