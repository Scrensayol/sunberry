local v1 = {}
local function tap(v2, v3)
	table.insert(v1, v2)
	return v3
end
local function clamp(v4, v5, v6)
	if v4 < v5 then
		return v5
	end
	if v6 < v4 then
		return v6
	end
	return v4
end
local function guard(v7)
	if not v7 then
		return
	end
	table.insert(v1, "guarded " .. tostring(v7))
end
local v8 = 0
local function bump(v9)
	v8 += v9
end
local v10 = tap("l", 1) + tap("r", 2) * tap("m", 3)
local v11 = tap("p", false) and tap("q", 1) or tap("z", 2)
local v12 = {}
v12[tap("ck", "count")] = (v12.count or 0) + tap("cv", 1)
for i = -2, 12, 5 do
	print(i, clamp(i, 0, 10))
	bump(i)
end
guard(nil)
guard("yes")
local function scaled(v13)
	return clamp(v13 * 2, 1, 9)
end
if clamp(v10, 0, 5) == 5 and scaled(3) > 4 then
	print("both")
end
while 0 < v8 do
	local v14 = clamp
	local v15 = v8
	v8 -= v14(v15, 1, 7)
	if v8 % 3 == 0 then
		break
	end
end
print(v10, v11, v12.count, v8, table.concat(v1, ","))
