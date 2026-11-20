local function v4(v1, v2)
	local v3
	for k, v in v1 do
		if v == v2 then
			v3 = k
			break
		end
	end
	if v3 == nil then
		return "none"
	end
	return v3
end
print(v4({1, 5, 3, 5}, 5), v4({1, 2}, 9), v4({}, 1))
local function v7(v5)
	local v6
	for i = 1, v5 do
		if i % 2 == 0 and 3 < i then
			v6 = i
			return v6
		end
	end
	return v6
end
print(v7(10), v7(3))
local function v12(v8, v9)
	local v10 = 0
	if v8 then
		for k2, v11 in v9 do
			if 2 < v11 then
				v10 = v11
				break
			end
		end
	else
		v10 = -1
	end
	return v10 * 10
end
print(v12(true, {1, 3, 4}), v12(true, {1}), v12(false, {9}))
local v13 = {}
for i2 = 1, 4 do
	for i3 = 1, 4 do
		if i2 < i3 then
			break
		end
		if (i2 + i3) % 2 ~= 1 then
			table.insert(v13, i2 * 10 + i3)
		end
	end
	if i2 ~= 3 then
		table.insert(v13, -i2)
	end
end
print(table.concat(v13, ","))
local function v17(v14)
	local v15 = 0
	local v16 = 0
	while v15 < v14 do
		v15 = v15 + 1
		if v15 % 3 ~= 0 then
			if 20 < v16 then
				break
			end
			v16 = v16 + v15
		end
	end
	return v15, v16
end
print(v17(5), v17(100))
local function v21(v18)
	local v19 = 0
	local v20 = {}
	repeat
		v19 = v19 + 1
		if v19 ~= 2 then
			table.insert(v20, v19)
		end
	until v18 <= v19
	return table.concat(v20, " ")
end
print(v21(5), v21(1))
local function v25(v22)
	for k3, v23 in v22 do
		for k4, v24 in v23 do
			if v24 < 0 then
				return true, v24
			end
		end
	end
	return false
end
print(v25({{1, 2}, {3, -4, -5}}), v25({{1}}))
local v26 = {}
for i4 = 1, 3 do
	local v27 = i4 * 2
	if i4 ~= 2 then
		local function v28()
			return i4 + v27
		end
		table.insert(v26, v28)
	end
end
local v29 = {}
for k5, v30 in v26 do
	table.insert(v29, v30())
end
print(table.concat(v29, ","))
local function v35(v31, v32)
	local v33 = 0
	for k6, v34 in v31 do
		v33 = v33 + 1
		if 10 < v34 then
			if v32 then
				print("big flag", v34)
				return v33
			end
			print("big", v34)
			return -v33
		end
	end
	return 0
end
local function v38(v36)
	for k7, v37 in v36 do
		if v37 == 1 then
			print("one")
			return
		end
		if v37 == 2 then
			print("two")
			return
		end
		print("other", v37)
	end
	print("end")
end
print(v35({1, 2, 30, 40}, true), v35({1, 50}, false), v35({1}, true))
v38({5, 2, 1})
v38({5, 6})
