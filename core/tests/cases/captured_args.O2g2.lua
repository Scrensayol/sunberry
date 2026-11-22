local n = 1
local function bump()
	n += 10
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
v1(v2, n, n)
local v3 = print
bump()
local v4 = n
local v5 = n
v3(math.max(v5, v4), n)
local v6 = print
bump()
v6(max(n, n), n)
local v7 = print
local v8 = n
bump()
v7(obj:show(v8, n, n))
local v9 = print
local v10 = tostring(n)
bump()
v9(v10 .. tostring(n), n)
local v11 = n
bump()
local t = {v11, n, n}
print(t[1], t[2], t[3])
local v12 = print
bump()
v12(n + n, n)
local v13 = print
local v14
v14 = select
local v15
v15 = 2
local v16 = n
bump()
v13(v14(v15, v16, n, n))
