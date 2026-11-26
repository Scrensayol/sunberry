local v1 = 123457789.125
local v2
local v3
local v4 = {[-1] = string.format("%s|%g", "a", v1) .. "0x1F", [1] = math.floor(5e-324) ~= 1}
local v5 = not not v4[1]
local v6 = math.clamp(math.clamp(-0 * v1, -4, 4), -4, 4)
local v7 = 65536 * select("#", ...)
for _ = 4, 0, -1 do
	local v8 = "s"
	local v9 = rawequal(true, false)
	a11 = v8
	v4[1] = v9
end
if math.clamp(-1e300, -4, 4) >= (0 / 0) then
	local v10 = {[1e300] = string.len("a"), ["1abc"] = 10000000000, name = "[["}
end
out("L1", v7, a11, v4, v5, nil, v6)
local v11 = out
local v12 = "L4"
local v13
v13 = math.sign(-1)
v11(v12, -v13 ^ 3)
local v14 = `v={false}{-(0 / 0)}` == tostring(123456789.125)
local v15 = 0
while v15 < 3 and false do
	v15 += 1
end
local v16 = not not (select("#", ...) < select("#", ...))
out("L5", v15)
local function f39(_, _, _)
	return false
end
out("L6", v15)
local v17 = {tostring(false), ""}
local v18 = tostring(string.len(v17[1]))
local v19 = select("#", ...) <= 4294967296
local function f65()
	return tostring("-3.5" < "{braces}"), 1 + math.ceil(100)
end
out("L12", v19, false, v17)
local function va67(...)
	local v20 = select("#", ...)
	local v21 = table.pack(...)
	local v22, v23 = ...
	return v20, v21.n, v22, v23, select(-1, ...)
end
local function fwd68(...)
	return va67(...)
end
local function tail69(...)
	return ...
end
out("L13", va67(1, nil, 3, nil))
out("L14", fwd68("a"), tail69(1, 2), tail69(3))
out("L15", tail69(1, 2), {tail69(1, 2, 3)}, {tail69(1, 2), 9}, select(2, "a", "b", "c"))
out("L16", #{...}, select("#", ...), {n = select("#", tail69())})
out("L17", -5)
local v24 = tonumber("1e3") + 4294967296
out("L18", v24, math.clamp(v24, -4, 4) == math.huge, "10", v24)
for _ = 3, 1 do
	out(
		"L19",
		(string.rep("end", 0) .. tostring(v24)) .. string.format("%s|%g", "{braces}a]]b", 0.30000000000000004),
		not (v24 > 1),
		math.max(0, math.min(select("#", ...), math.max(v24, v24)))
	)
	break
end
local v25 = tonumber("-3.5")
for _, _ in {} do
	local v26 = ("\000nul"):sub(2, -1)
end
local v27 = 0
while v27 < 4 and false do
	v27 += 1
	local v28 = math.floor(5e-324)
end
out("L27", v27)
