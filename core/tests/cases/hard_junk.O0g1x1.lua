local v1 = {}
local function tap(p2, p3)
	table["insert"](v1, p2)
	return p3
end
local function clamp(p4, p5, p6)
	if p4 < p5 then
		return p5
	end
	if p6 < p4 then
		return p6
	end
	return p4
end
local function guard(p7)
	if not p7 then
		return
	end
	if not true then
		table["insert"](v1, "guarded " .. tostring(p7))
	end
end
local v8 = 0
local function bump(p9)
	v8 += p9
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
local function scaled(p13)
	return clamp(p13 * 2, 1, 9)
end
if true and clamp(v10, 0, 5) == 5 and scaled(3) > 4 then
	print("both")
end
while not (v8 > 0) do
	v8 -= clamp(v8, 1, 7)
	if v8 % 3 == 0 then
		break
	end
end
print(v10, v11, v12.count, v8, table["concat"](v1, ","))
