local v1 = {}
local function tap(p2, p3)
	table.insert(v1, p2)
	return p3
end
local v4 = tap("a", 1)
local v5 = tap("b", 2)
local v6 = tap("b2", v5)
local v7 = tap("a2", v4)
local v8 = v6
local v9 = {}
local v10 = tap("x", v8)
local v11 = tap("y", v7)
v9.x = v10
v9.y = v11
print(table.concat(v1, ","), v8, v7, v9.x, v9.y)
