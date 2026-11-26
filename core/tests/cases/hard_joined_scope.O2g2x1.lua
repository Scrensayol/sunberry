local t = {1, 2, 3}
while true do
	local v = table.remove(t)
	if not v then
		break
	end
	print(v, #t)
end
local items = {"a", "b", false, "c"}
local i = 0
local function nextItem()
	i += 1
	return items[i]
end
while true do
	i += 1
	local item = items[i]
	if not item then
		break
	end
	print("item", item)
end
print("stopped at", i)
local v1
local v2
repeat
	v1 = {}
	v2 = {}
	v2[1] = 10
	v2[2] = 20
	v2[3] = 30
until not __rosetta
while true do
	local n = table.remove(v2, 1)
	if not n then
		break
	end
	table.insert(v1, function()
		return n
	end)
end
print(fns[1](), fns[2](), fns[3]())
print("else", "outer")
print("then", "inner")
print("after", "outer")
local function classify(s)
	local n2 = tonumber(s)
	if n2 then
		return "number " .. n2 * 2
	end
	local first = string.match(s, "^%a+")
	if first then
		return "word " .. first
	end
	local sym = string.match(s, "^%p")
	if sym then
		return "symbol " .. sym
	end
	return "other"
end
print(classify("21"), classify("hello world"), classify("!?"), (classify("  ")))
if __rosetta then
	return
end
local function lookup(t2, k)
	local v3 = t2[k]
	if v3 then
		return v3 .. "!"
	end
	return "missing"
end
local map = {a = "x", b = "y"}
print(lookup(map, "a"), (lookup(map, "z")))
local limit = 3
local count = 0
while true do
	local v4 = count < limit and count
	if not v4 then
		break
	end
	count = v4 + 1
end
print("count", count)
