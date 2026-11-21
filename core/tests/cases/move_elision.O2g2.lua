local log = {}
local function tap(name, v)
	table.insert(log, name)
	return v
end
local a = tap("a", 1)
local b = tap("b", 2)
local v1 = tap("b2", b)
local b2 = tap("a2", a)
a = v1
local t = {}
local v2 = tap("x", a)
local v3 = tap("y", b2)
t.x = v2
t.y = v3
print(table.concat(log, ","), a, b2, t.x, t.y)
