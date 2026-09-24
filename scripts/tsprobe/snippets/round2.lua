local M = {}
M.a = function() end
local x = { [1] = function() end }
function M.outer() local function inner() end; return inner end
print(#M, M[1], M["a"], M.a)
goto done
::done::
break_me = [==[ level two ]==]
