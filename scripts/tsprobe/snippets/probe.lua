local m = require("mod.sub")
require "other"
local cfg = dofile("x.lua")
-- line comment
--[[ block
comment ]]
--- doc comment (LDoc)
local M = {}
local function helper(a, b) return a and b or not a end
function M.method(self, x) self.x = x; return self:other() end
function M:colon(x) return x end
function global_fn() end
local anon = function(a) return a end
local t = { f = function() end, ["k"] = 1, g = function(z) return z end }
a, b = function() end, function() end
function main(argc)
  if argc > 1 then print("x") elseif argc == 0 then print("y") else print("z") end
  for i = 1, 3 do if i == 1 then goto continue end ::continue:: end
  for k, v in pairs(t) do end
  while argc > 0 do argc = argc - 1 end
  repeat argc = argc + 1 until argc > 3
  local s = 'single' .. "double" .. [[long]] .. 42 .. 3.5
  local function inner() return helper(1, 2) end
  return helper(argc, 1), M.method(nil, 1), m.f(), t.f()
end
return M
