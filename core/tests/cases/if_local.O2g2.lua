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
local fns = {}
local queue = {10, 20, 30}
while true do
	local n = table.remove(queue, 1)
	if not n then
		break
	end
	table.insert(fns, function()
		return n
	end)
end
print(fns[1](), fns[2](), fns[3]())
local x = "outer"
print("else", "outer")
local x2 = "inner"
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
local function lookup(t2, k)
	local v2 = t2[k]
	if v2 then
		return v2 .. "!"
	end
	return "missing"
end
local map = {a = "x", b = "y"}
print(lookup(map, "a"), (lookup(map, "z")))
local limit = 3
local count = 0
while true do
	local v3 = count < limit and count
	if not v3 then
		break
	end
	count = v3 + 1
end
print("count", count)
