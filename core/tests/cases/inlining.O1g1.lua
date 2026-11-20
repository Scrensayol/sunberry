local v1 = {}
local function v4(v2, v3)
	table.insert(v1, v2)
	return v3
end
local function v8(v5, v6, v7)
	if v5 < v6 then
		return v6
	end
	if v7 < v5 then
		return v7
	end
	return v5
end
local function v10(v9)
	if not v9 then
		return
	end
	table.insert(v1, "guarded " .. tostring(v9))
end
local v11 = 0
local function v13(v12)
	v11 = v11 + v12
end
local v14 = v4("l", 1) + v4("r", 2) * v4("m", 3)
local v15 = v4("p", false) and v4("q", 1) or v4("z", 2)
local v16 = {}
v16[v4("ck", "count")] = (v16.count or 0) + v4("cv", 1)
for i = -2, 12, 5 do
	print(i, v8(i, 0, 10))
	v13(i)
end
v10(nil)
v10("yes")
local function v18(v17)
	return v8(v17 * 2, 1, 9)
end
if v8(v14, 0, 5) == 5 and v18(3) > 4 then
	print("both")
end
while 0 < v11 do
	local v19 = v8
	local v20 = v11
	v11 = v11 - v19(v20, 1, 7)
	if v11 % 3 == 0 then
		break
	end
end
print(v14, v15, v16.count, v11, table.concat(v1, ","))
