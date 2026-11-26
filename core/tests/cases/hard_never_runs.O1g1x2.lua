local v1 = 123457789.125
local v2 = {[-1] = string.format("%s|%g", "a", v1) .. "0x1F", [1] = math.floor(5e-324) ~= 1}
local v3 = not not v2[1]
local v4 = math.clamp(math.clamp(-0 * v1, -4, 4), -4, 4)
local v5 = 65536 * select("#", ...)
for _ = 4, 0, -1 do
	local v6 = "s"
	local v7 = rawequal(true, false)
	a11 = v6
	v2[1] = v7
end
if math.clamp(-1e300, -4, 4) >= (0 / 0) then
	local v8 = {[1e300] = string.len("a"), ["1abc"] = 10000000000, name = "[["}
end
out("L1", v5, a11, v2, v3, nil, v4)
local v9 = out
local v10 = "L4"
local v11
v11 = math.sign(-1)
v9(v10, -v11 ^ 3)
if `v={false}{-(0 / 0)}` ~= tostring(123456789.125) then
end
local v12 = 0
if v12 < 3 then
end
local v13 = not not (select("#", ...) < select("#", ...))
out("L5", v12)
local function f39(_, _, _)
	return false
end
out("L6", v12)
local v14 = {tostring(false), ""}
local v15 = tostring(string.len(v14[1]))
local v16 = select("#", ...) <= 4294967296
local function f65()
	return tostring("-3.5" < "{braces}"), 1 + math.ceil(100)
end
out("L12", v16, false, v14)
local function va67(...)
	local v17 = select("#", ...)
	local v18 = table.pack(...)
	local v19, v20 = ...
	return v17, v18.n, v19, v20, select(-1, ...)
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
local v21 = tonumber("1e3") + 4294967296
out("L18", v21, math.clamp(v21, -4, 4) == math.huge, "10", v21)
for _ = 3, 1 do
	out(
		"L19",
		(string.rep("end", 0) .. tostring(v21)) .. string.format("%s|%g", "{braces}a]]b", 0.30000000000000004),
		not (v21 > 1),
		math.max(0, math.min(select("#", ...), math.max(v21, v21)))
	)
	break
end
local v22 = tonumber("-3.5")
for _, _ in {} do
	local v23 = ("\000nul"):sub(2, -1)
end
local v24 = 0
if v24 < 4 then
end
out("L27", v24)
