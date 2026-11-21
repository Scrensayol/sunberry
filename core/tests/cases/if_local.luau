-- Luau's experimental `if local` / `while local` / `const` syntax (a friend's case). The binding
-- lives only in the then-branch or loop body, so an outer variable with the same name has to
-- stay visible in the else-branch, and every loop iteration gets a fresh local for closures.
do
	const t = {1, 2, 3}
	while local v = table.remove(t) do
		print(v, #t)
	end
end
do
	local items = {"a", "b", false, "c"}
	local i = 0
	local function nextItem()
		i += 1
		return items[i]
	end
	-- false ends the loop just like nil
	while const item = nextItem() do
		print("item", item)
	end
	print("stopped at", i)
end
do
	local fns = {}
	local queue = {10, 20, 30}
	while local n = table.remove(queue, 1) do
		table.insert(fns, function()
			return n
		end)
	end
	print(fns[1](), fns[2](), fns[3]())
end
do
	local x = "outer"
	if local x = nil then
		print("then", x)
	else
		print("else", x)
	end
	if local x = "inner" then
		print("then", x)
	end
	print("after", x)
end
do
	local function classify(s)
		if local n: number? = tonumber(s) then
			return "number " .. n * 2
		elseif local first = string.match(s, "^%a+") then
			return "word " .. first
		elseif const sym = string.match(s, "^%p") then
			return "symbol " .. sym
		else
			return "other"
		end
	end
	print(classify("21"), classify("hello world"), classify("!?"), classify("  "))
end
do
	local function lookup(t, k)
		return if local v = t[k] then v .. "!" else "missing"
	end
	local map = {a = "x", b = "y"}
	print(lookup(map, "a"), lookup(map, "z"))
	const limit = 3
	local count = 0
	while local v = count < limit and count do
		count = v + 1
	end
	print("count", count)
end
