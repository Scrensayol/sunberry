local function find(p1, p2) -- name: find
	local v3 = nil
	for v4, v5 in p1 do
		if v5 == p2 then
			v3 = v4
			break
		end
	end
	return v3 == nil and "none" or v3
end
print(find({
	1,
	5,
	3,
	5
}, 5), find({ 1, 2 }, 9), find({}, 1))
local function first_even(p6) -- name: first_even
	local v7 = nil
	for v8 = 1, p6 do
		if v8 % 2 == 0 and v8 > 3 then
			return v8
		end
	end
	return v7
end
print(first_even(10), first_even(3))
local function f(p9, p10) -- name: f
	local v11 = 0
	if p9 then
		for _, v12 in p10 do
			if v12 > 2 then
				v11 = v12
				break
			end
		end
	else
		v11 = -1
	end
	return v11 * 10
end
print(f(true, { 1, 3, 4 }), f(true, { 1 }), f(false, { 9 }))
local v13 = {}
for v14 = 1, 4 do
	for v15 = 1, 4 do
		if v14 < v15 then
			break
		end
		if (v14 + v15) % 2 == 1 then
			continue
		end
		local v16 = v14 * 10 + v15
		table.insert(v13, v16)
	end
	if v14 == 3 then
		continue
	end
	local v17 = -v14
	table.insert(v13, v17)
end
print(table.concat(v13, ","))
local function g(p18) -- name: g
	local v19 = 0
	local v20 = 0
	while v19 < p18 do
		v19 += 1
		if v19 % 3 == 0 then
			continue
		end
		if v20 > 20 then
			break
		end
		v20 += v19
	end
	return v19, v20
end
print(g(5), g(100))
local function h(p21) -- name: h
	local v22 = 0
	local v23 = {}
	while true do
		v22 += 1
		if v22 ~= 2 then
			table.insert(v23, v22)
		end
		if p21 <= v22 then
			return table.concat(v23, " ")
		end
	end
end
print(h(5), h(1))
local function any_neg(p24) -- name: any_neg
	for _, v25 in p24 do
		for _, v26 in v25 do
			if v26 < 0 then
				return true, v26
			end
		end
	end
	return false
end
print(any_neg({
	{ 1, 2 },
	{ 3, -4, -5 }
}), any_neg({
	{ 1 }
}))
local v27 = {}
for v_u_28 = 1, 3 do
	local v_u_29 = v_u_28 * 2
	if v_u_28 == 2 then
		continue
	end
	table.insert(v27, function()
		-- upvalues: (copy) v_u_28, (copy) v_u_29
		return v_u_28 + v_u_29
	end)
end
local v30 = {}
for _, v31 in v27 do
	table.insert(v30, v31())
end
print(table.concat(v30, ","))
local function f_2(p32, p33) -- name: f
	local v34 = 0
	for _, v35 in p32 do
		v34 += 1
		if v35 > 10 then
			if p33 then
				print("big flag", v35)
				return v34
			else
				print("big", v35)
				return -v34
			end
		end
	end
	return 0
end
local function g_2(p36) -- name: g
	for _, v37 in p36 do
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
print(f_2({
	1,
	2,
	30,
	40
}, true), f_2({ 1, 50 }, false), f_2({ 1 }, true))
g_2({ 5, 2, 1 })
g_2({ 5, 6 })