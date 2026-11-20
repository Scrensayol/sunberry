local v1 = 1
local function v2()
	v1 = v1 + 10
	return v1
end
local max
max = math.max
local v4 = {
	show = function(v3, ...)
		return select("#", ...), ...
	end
}
print(v1, v2(), v1)
local v5 = print
local v6 = v1
v5(math.max(v6, v2()), v1)
local v7 = print
local v8 = v1
v7(max(v8, v2()), v1)
print(v4:show(v1, v2(), v1))
local v9 = print
local v10 = v1
v9(tostring(v10) .. tostring(v2()), v1)
local v11 = {v1, v2(), v1}
print(v11[1], v11[2], v11[3])
print(v1 + v2(), v1)
local v12 = print
local v13
v13 = select
local v14
v14 = 2
local v15 = v1
v12(v13(v14, v15, v2(), v1))
