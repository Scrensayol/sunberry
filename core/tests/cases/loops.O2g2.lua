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
		if i2 % 2 == 0 and i2 > 3 then
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
			if v2 > 2 then
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
local v3 = f(true, {1, 3, 4})
local v4 = f(true, {1})
v1(v3, v4, (f()))
local out = {}
local v5 = {9}
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
		i4 += 1
		if i4 % 3 ~= 0 then
			if s > 20 then
				break
			end
			s += i4
		end
	end
	return i4, s
end
print(g(5), g(100))
local function h(n3)
	local i5 = 0
	local acc = {}
	repeat
		i5 += 1
		if i5 ~= 2 then
			table.insert(acc, i5)
		end
	until n3 <= i5
	return table.concat(acc, " ")
end
print(h(5), h(1))
local function any_neg(t3)
	for _, row in t3 do
		for _, v6 in row do
			if v6 < 0 then
				return true, v6
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
		table.insert(fs, function()
			return i6 + j2
		end)
	end
end
local r2 = {}
for _, f2 in fs do
	table.insert(r2, f2())
end
print(table.concat(r2, ","))
local function f3(list, flag)
	local n4 = 0
	for _, v7 in list do
		n4 += 1
		if v7 > 10 then
			if flag then
				print("big flag", v7)
				return n4
			end
			print("big", v7)
			return -n4
		end
	end
	return 0
end
local function g2(list2)
	for _, v8 in list2 do
		if v8 == 1 then
			print("one")
			return
		end
		if v8 == 2 then
			print("two")
			return
		end
		print("other", v8)
	end
	print("end")
end
local v9 = print
local list3 = {1, 2, 30, 40}
local n5 = 0
local v10 = false
local v11
for _, v12 in list3 do
	n5 += 1
	if v12 > 10 then
		print("big flag", v12)
		v11 = n5
		v10 = true
		break
	end
end
if not v10 then
	v11 = 0
end
local v13 = f3({1, 50})
local list4 = {1}
local n6 = 0
local v14 = false
local v15
for _, v16 in list4 do
	n6 += 1
	if v16 > 10 then
		print("big flag", v16)
		v15 = n6
		v14 = true
		break
	end
end
if not v14 then
	v15 = 0
end
v9(v11, v13, v15)
g2({5, 2, 1})
g2({5, 6})
