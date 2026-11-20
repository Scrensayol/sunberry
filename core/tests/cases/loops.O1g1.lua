local function find(v1, v2)
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
print(find({1, 5, 3, 5}, 5), find({1, 2}, 9), find({}, 1))
local function first_even(v4)
	local v5
	for i = 1, v4 do
		if i % 2 == 0 and 3 < i then
			v5 = i
			return v5
		end
	end
	return v5
end
print(first_even(10), first_even(3))
local function f(v6, v7)
	local v8 = 0
	if v6 then
		for _, v9 in v7 do
			if 2 < v9 then
				v8 = v9
				break
			end
		end
	else
		v8 = -1
	end
	return v8 * 10
end
print(f(true, {1, 3, 4}), f(true, {1}), f(false, {9}))
local v10 = {}
for i2 = 1, 4 do
	for i3 = 1, 4 do
		if i2 < i3 then
			break
		end
		if (i2 + i3) % 2 ~= 1 then
			table.insert(v10, i2 * 10 + i3)
		end
	end
	if i2 ~= 3 then
		table.insert(v10, -i2)
	end
end
print(table.concat(v10, ","))
local function g(v11)
	local v12 = 0
	local v13 = 0
	while v12 < v11 do
		v12 += 1
		if v12 % 3 ~= 0 then
			if 20 < v13 then
				break
			end
			v13 += v12
		end
	end
	return v12, v13
end
print(g(5), g(100))
local function h(v14)
	local v15 = 0
	local v16 = {}
	repeat
		v15 += 1
		if v15 ~= 2 then
			table.insert(v16, v15)
		end
	until v14 <= v15
	return table.concat(v16, " ")
end
print(h(5), h(1))
local function any_neg(v17)
	for _, v18 in v17 do
		for _, v19 in v18 do
			if v19 < 0 then
				return true, v19
			end
		end
	end
	return false
end
print(any_neg({{1, 2}, {3, -4, -5}}), any_neg({{1}}))
local v20 = {}
for i4 = 1, 3 do
	local v21 = i4 * 2
	if i4 ~= 2 then
		local function v22()
			return i4 + v21
		end
		table.insert(v20, v22)
	end
end
local v23 = {}
for _, v24 in v20 do
	table.insert(v23, v24())
end
print(table.concat(v23, ","))
local function f2(v25, v26)
	local v27 = 0
	for _, v28 in v25 do
		v27 += 1
		if 10 < v28 then
			if v26 then
				print("big flag", v28)
				return v27
			end
			print("big", v28)
			return -v27
		end
	end
	return 0
end
local function g2(v29)
	for _, v30 in v29 do
		if v30 == 1 then
			print("one")
			return
		end
		if v30 == 2 then
			print("two")
			return
		end
		print("other", v30)
	end
	print("end")
end
print(f2({1, 2, 30, 40}, true), f2({1, 50}, false), f2({1}, true))
g2({5, 2, 1})
g2({5, 6})
