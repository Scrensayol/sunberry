local a1
local v3 = 123457789.125
local v4
local v5
local r9 = {[-1] = string.format("%s|%g", "a", v3) .. "0x1F", [1] = true}
v5 = not not r9[1]
v3 = math.clamp(math.clamp(v3 * -0, -4, 4), -4, 4)
local a13 = 65536 * select("#", ...)
local v1 = "s"
local v2 = rawequal(true, false)
a11 = v1
r9[1] = v2
local v6 = "s"
local v7 = rawequal(true, false)
a11 = v6
r9[1] = v7
local v8 = "s"
local v9 = rawequal(true, false)
a11 = v8
r9[1] = v9
local v10 = "s"
local v11 = rawequal(true, false)
a11 = v10
r9[1] = v11
local v12 = "s"
local v13 = rawequal(true, false)
a11 = v12
r9[1] = v13
out("L1", a13, a11, r9, v5, nil, v3)
out("L4", 1)
local w25 = `v={false}{-(0 / 0)}` == tostring(123456789.125)
local w252 = 0
while w252 < 3 and false do
	w252 += 1
end
local b29 = not not (select("#", ...) < select("#", ...))
out("L5", w252)
local function f39(_, _, _)
	local p31 = true
	local p312 = true
	return false
end
out("L6", w252)
local t58 = {tostring(false), ""}
local s60 = tostring((string.len(t58[1])))
local a62 = false
local s602 = true
s602 = select("#", ...) <= 4294967296
local function f65()
	return tostring("-3.5" < "{braces}"), 101
end
out("L12", s602, false, t58)
local function va67(...)
	local n = select("#", ...)
	local t = table.pack(...)
	local first, second = ...
	return n, t.n, first, second, select(-1, ...)
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
local n70 = tonumber("1e3") + 4294967296
out("L18", n70, math.clamp(n70, -4, 4) == (1 / 0), "10", n70)
n70 = tonumber("-3.5")
for _, _ in {} do
	local s80 = ("\000nul"):sub(2, -1)
end
local w127 = 0
while w127 < 4 and false do
	w127 += 1
end
out("L27", w127)
