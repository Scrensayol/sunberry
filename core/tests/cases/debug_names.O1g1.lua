local function v7(v1, v2, v3, v4)
	local v5 = v3 - v1
	local v6 = v4 - v2
	return math.sqrt(v5 * v5 + v6 * v6)
end
local v8 = {{0, 0}, {3, 4}, {6, 8}}
local v9 = 0
for i = 2, #v8 do
	local v10 = v8[i - 1]
	local v11 = v8[i]
	v9 = v9 + v7(v10[1], v10[2], v11[1], v11[2])
end
print(v9)
