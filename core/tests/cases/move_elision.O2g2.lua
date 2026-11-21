local log = {}
local function tap(name, v)
	table.insert(log, name)
	return v
end
local a = tap("a", 1)
local b = tap("b", 2)
local v1 = tap("b2", b)
b = tap("a2", a)
a = v1
local t = {}
local v2 = tap("x", a)
local v3 = tap("y", b)
t.x = v2
t.y = v3
print(table.concat(log, ","), a, b, t.x, t.y)
