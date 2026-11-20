local function find(p1, p2)
	local v3
	for k, v in p1 do
		if v == p2 then
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
local function first_even(p4)
	local v5
	for i = 1, p4 do
		if i % 2 == 0 and i > 3 then
			v5 = i
			return v5
		end
	end
	return v5
end
print(first_even(10), first_even(3))
local function f(p6, p7)
	local v8 = 0
	if p6 then
		for _, v2 in p7 do
			if v2 > 2 then
				v8 = v2
				break
			end
		end
	else
		v8 = -1
	end
	return v8 * 10
end
print(f(true, {1, 3, 4}), f(true, {1}), f(false, {9}))
local v9 = {}
for i2 = 1, 4 do
	for i3 = 1, 4 do
		if i2 < i3 then
			break
		end
		if (i2 + i3) % 2 ~= 1 then
			table.insert(v9, i2 * 10 + i3)
		end
	end
	if i2 ~= 3 then
		table.insert(v9, -i2)
	end
end
print(table.concat(v9, ","))
local function g(p10)
	local v11 = 0
	local v12 = 0
	while v11 < p10 do
		v11 += 1
		if v11 % 3 ~= 0 then
			if v12 > 20 then
				break
			end
			v12 += v11
		end
	end
	return v11, v12
end
print(g(5), g(100))
local function h(p13)
	local v14 = 0
	local v15 = {}
	repeat
		v14 += 1
		if v14 ~= 2 then
			table.insert(v15, v14)
		end
	until p13 <= v14
	return table.concat(v15, " ")
end
print(h(5), h(1))
local function any_neg(p16)
	for _, v4 in p16 do
		for _, v6 in v4 do
			if v6 < 0 then
				return true, v6
			end
		end
	end
	return false
end
print(any_neg({{1, 2}, {3, -4, -5}}), any_neg({{1}}))
local v17 = {}
for i4 = 1, 3 do
	local v18 = i4 * 2
	if i4 ~= 2 then
		local function v19()
			return i4 + v18
		end
		table.insert(v17, v19)
	end
end
local v20 = {}
for _, v7 in v17 do
	table.insert(v20, v7())
end
print(table.concat(v20, ","))
local function f2(p21, p22)
	local v23 = 0
	for _, v10 in p21 do
		v23 += 1
		if v10 > 10 then
			if p22 then
				print("big flag", v10)
				return v23
			end
			print("big", v10)
			return -v23
		end
	end
	return 0
end
local function g2(p24)
	for _, v13 in p24 do
		if v13 == 1 then
			print("one")
			return
		end
		if v13 == 2 then
			print("two")
			return
		end
		print("other", v13)
	end
	print("end")
end
print(f2({1, 2, 30, 40}, true), f2({1, 50}, false), f2({1}, true))
g2({5, 2, 1})
g2({5, 6})
