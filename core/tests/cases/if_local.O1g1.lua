local v1 = {1, 2, 3}
while true do
	local v2 = table.remove(v1)
	if not v2 then
		break
	end
	print(v2, #v1)
end
local v3 = {"a", "b", false, "c"}
local v4 = 0
local function nextItem()
	v4 += 1
	return v3[v4]
end
while true do
	local v5 = nextItem()
	if not v5 then
		break
	end
	print("item", v5)
end
print("stopped at", v4)
local v6 = {}
local v7 = {10, 20, 30}
while true do
	local v8 = table.remove(v7, 1)
	if not v8 then
		break
	end
	table.insert(v6, function()
		return v8
	end)
end
print(v6[1](), v6[2](), v6[3]())
print("else", "outer")
local v9 = "inner"
print("then", "inner")
print("after", "outer")
local function classify(p10)
	local v11 = tonumber(p10)
	if v11 then
		return "number " .. v11 * 2
	end
	local v12 = string.match(p10, "^%a+")
	if v12 then
		return "word " .. v12
	end
	local v13 = string.match(p10, "^%p")
	if v13 then
		return "symbol " .. v13
	end
	return "other"
end
print(classify("21"), classify("hello world"), classify("!?"), classify("  "))
local function lookup(p14, p15)
	local v16 = p14[p15]
	if v16 then
		return v16 .. "!"
	end
	return "missing"
end
local v17 = {a = "x", b = "y"}
print(lookup(v17, "a"), lookup(v17, "z"))
local v18 = 0
while true do
	local v19 = v18 < 3 and v18
	if not v19 then
		break
	end
	v18 = v19 + 1
end
print("count", v18)
