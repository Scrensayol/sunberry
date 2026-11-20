local function distance(v1, v2, v3, v4)
	local v5 = v3 - v1
	local v6 = v4 - v2
	return math.sqrt(v5 * v5 + v6 * v6)
end
local v7 = {{0, 0}, {3, 4}, {6, 8}}
local v8 = 0
for i = 2, #v7 do
	local v9 = v7[i - 1]
	local v10 = v7[i]
	v8 += distance(v9[1], v9[2], v10[1], v10[2])
end
print(v8)
