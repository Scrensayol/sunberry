local function find(t, x)
	local found
	for i, v in t do
		if v == x then
			found = i
			break
		end
	end
	if found == nil then
		return "none"
	end
	return found
end
print(find({1, 5, 3, 5}, 5), find({1, 2}, 9), (find({}, 1)))
local function first_even(n)
	local r
	for i2 = 1, n do
		if i2 % 2 == 0 and 3 < i2 then
			r = i2
			return r
		end
	end
	return r
end
print(first_even(10), nil)
local function f(a, t2)
	local hit = 0
	if a then
		for _, v2 in t2 do
			if 2 < v2 then
				hit = v2
				break
			end
		end
	else
		hit = -1
	end
	return hit * 10
end
local v1 = print
local t3 = {1, 3, 4}
local hit2 = 0
for _2, v3 in t3 do
	if 2 < v3 then
		hit2 = v3
		break
	end
end
local v4 = hit2 * 10
local t4 = {1}
local hit3 = 0
for _3, v5 in t4 do
	if 2 < v5 then
		hit3 = v5
		break
	end
end
local v6 = hit3 * 10
v1(v4, v6, -1 * 10)
local out = {}
local v7 = {9}
for i3 = 1, 4 do
	for j = 1, 4 do
		if i3 < j then
			break
		end
		if (i3 + j) % 2 ~= 1 then
			table.insert(out, i3 * 10 + j)
		end
	end
	if i3 ~= 3 then
		table.insert(out, -i3)
	end
end
print(table.concat(out, ","))
local function g(n2)
	local i4 = 0
	local s = 0
	while i4 < n2 do
		i4 = i4 + 1
		if i4 % 3 ~= 0 then
			if 20 < s then
				break
			end
			s = s + i4
		end
	end
	return i4, s
end
print(g(5), g(100))
local function h(n3)
	local i5 = 0
	local acc = {}
	repeat
		i5 = i5 + 1
		if i5 ~= 2 then
			table.insert(acc, i5)
		end
	until n3 <= i5
	return table.concat(acc, " ")
end
print(h(5), h(1))
local function any_neg(t5)
	for _4, row in t5 do
		for _5, v8 in row do
			if v8 < 0 then
				return true, v8
			end
		end
	end
	return false
end
print(any_neg({{1, 2}, {3, -4, -5}}), any_neg({{1}}))
local fs = {}
for i6 = 1, 3 do
	local j2 = i6 * 2
	if i6 ~= 2 then
		local function v9()
			return i6 + j2
		end
		table.insert(fs, v9)
	end
end
local r2 = {}
for _6, f2 in fs do
	table.insert(r2, f2())
end
print(table.concat(r2, ","))
local function f3(list, flag)
	local n4 = 0
	for _7, v10 in list do
		n4 = n4 + 1
		if 10 < v10 then
			if flag then
				print("big flag", v10)
				return n4
			end
			print("big", v10)
			return -n4
		end
	end
	return 0
end
local function g2(list2)
	for _8, v11 in list2 do
		if v11 == 1 then
			print("one")
			return
		end
		if v11 == 2 then
			print("two")
			return
		end
		print("other", v11)
	end
	print("end")
end
local v12 = print
local list3 = {1, 2, 30, 40}
local n5 = 0
local v13 = false
local v14
for _9, v15 in list3 do
	n5 = n5 + 1
	if 10 < v15 then
		print("big flag", v15)
		v14 = n5
		v13 = true
		break
	end
end
if not v13 then
	v14 = 0
end
local list4 = {1, 50}
local n6 = 0
local v16 = false
local v17
for _10, v18 in list4 do
	n6 = n6 + 1
	if 10 < v18 then
		print("big", v18)
		v17 = -n6
		v16 = true
		break
	end
end
if not v16 then
	v17 = 0
end
local list5 = {1}
local n7 = 0
local v19 = false
local v20
for _11, v21 in list5 do
	n7 = n7 + 1
	if 10 < v21 then
		print("big flag", v21)
		v20 = n7
		v19 = true
		break
	end
end
if not v19 then
	v20 = 0
end
v12(v14, v17, v20)
g2({5, 2, 1})
g2({5, 6})
