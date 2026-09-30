-- Flow sample (analysis-track §5.3, round 3): every statement,
-- declaration and access shape the flow/1 lowering table names, in
-- Lua 5.4.

local M = {}

counter = 0

function M.branches(a, b, ...)
  local label
  if a > 0 then
    label = "pos"
  elseif a < 0 then
    label = "neg"
  else
    label = "zero"
  end
  if b then
  end
  local total = 0
  for i = 1, b, 1 do
    if i == 3 then
      goto continue
    end
    if i > 7 then
      break
    end
    total = total + i
    ::continue::
  end
  for key, value in pairs({ ... }) do
    total = total + key + value
  end
  local n = 3
  while n > 0 do
    n = n - 1
  end
  while true do
    if total > 10 then
      break
    end
    total = total + 1
  end
  repeat
    n = n + 1
  until n > 2
  repeat
    n = n + 1
    if n > 5 then
      break
    end
  until false
  repeat
    n = n - 1
    if n < 0 then
      break
    end
  until nil
  counter = counter + total
  return label, total
end

function M:method(x)
  self.last = x
  return self.last
end

local function accesses(t, flag)
  local x, y = 0, 0
  x = x + 1
  t.field = x
  t[1] = x
  t.nested.deep = y
  local z = flag and x or y
  local shadow = 1
  do
    local shadow = 2
    shadow = shadow + 1
  end
  global_value = shadow
  local a <const> = 5
  local named = { key = a }
  return z + a + named.key
end

local function closures(values)
  local base = 10
  local scale = 2
  local function helper(v)
    scale = scale + 1
    return v * base
  end
  local add = function(d)
    return d + base
  end
  return helper(values[1]) + add(scale)
end

local function dynamic(code)
  local chunk = load(code)
  local info = debug.getinfo(1)
  return chunk, info
end

local function fatal(code)
  if code then
    error("fatal")
  end
  os.exit(1)
  local after = 1
  return after
end

M.accesses, M.closures, M.dynamic, M.fatal = accesses, closures, dynamic, fatal

return M
