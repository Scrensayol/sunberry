local n = 1
local function bump()
	n = n + 10
	return n
end
local max
max = math.max
local obj = {}
function obj:show(...)
	return select("#", ...), ...
end
local v1 = print
local v2 = n
bump()
local v3 = n
v1(v2, v3, n)
local v4 = print
bump()
local v5 = n
local v6 = n
v4(math.max(v6, v5), n)
local v7 = print
bump()
local v8 = n
local v9 = n
v7(max(v9, v8), n)
local v10 = print
local v11 = n
bump()
v10(obj:show(v11, n, n))
local v12 = print
local v13 = n
local v14 = tostring(v13)
bump()
local v15 = n
v12(v14 .. tostring(v15), n)
local v16 = n
bump()
local t = {v16, n, n}
print(t[1], t[2], t[3])
local v17 = print
bump()
v17(n + n, n)
local v18 = print
local v19
v19 = select
local v20
v20 = 2
local v21 = n
bump()
local v22 = n
v18(v19(v20, v21, v22, n))
