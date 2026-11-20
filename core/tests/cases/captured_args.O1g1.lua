local v1 = 1
local function bump()
	v1 += 10
	return v1
end
local max
max = math.max
local v2 = {
	show = function(_, ...)
		return select("#", ...), ...
	end
}
print(v1, bump(), v1)
local v3 = print
local v4 = v1
v3(math.max(v4, bump()), v1)
local v5 = print
local v6 = v1
v5(max(v6, bump()), v1)
print(v2:show(v1, bump(), v1))
local v7 = print
local v8 = v1
v7(tostring(v8) .. tostring(bump()), v1)
local v9 = {v1, bump(), v1}
print(v9[1], v9[2], v9[3])
print(v1 + bump(), v1)
local v10 = print
local v11
v11 = select
local v12
v12 = 2
local v13 = v1
v10(v11(v12, v13, bump(), v1))
