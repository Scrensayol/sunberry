local log = {}
local function tap(name, v)
	table.insert(log, name)
	return v
end
local a = tap("a", 1)
local b = tap("b", 2)
local v2 = b
tap("b2")
local b2 = a
a = tap("a2", v2)
local v3 = a
tap("x")
local v4 = b2
tap("y")
local t = {x = v3, y = v4}
print(table.concat(log, ","), a, b2, t.x, t.y)
