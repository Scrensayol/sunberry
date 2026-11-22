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
print(tostring(v1) .. tostring(bump()), v1)
local v7 = {v1, bump(), v1}
print(v7[1], v7[2], v7[3])
print(v1 + bump(), v1)
local v8 = print
local v9
v9 = select
local v10
v10 = 2
local v11 = v1
v8(v9(v10, v11, bump(), v1))
