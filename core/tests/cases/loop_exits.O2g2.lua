local v1 = {}
local function v2()
	return {[2] = {seen = true}}
end
v1.get = v2
local function seen(id)
	local e = ({[2] = {seen = true}})[id]
	return if e == nil then false else e.seen ~= nil
end
local function pick(tips)
	local index
	for i, tip in tips do
		if tip.show then
			local id2 = tip.id
			local e2 = ({[2] = {seen = true}})[id2]
			if not (if e2 == nil then false else e2.seen ~= nil) then
				index = i
				break
			end
		end
	end
	if index == nil then
		return false
	end
	local ok, v = pcall(function()
		return tips[index].id * 100
	end)
	return ok, v
end
print(pick({
	{show = false, id = 1},
	{show = true, id = 2},
	{show = true, id = 3},
	{show = true, id = 4}
}))
print(pick({{show = true, id = 2}}))
local function nextShown(tips2, from)
	local index2
	for i2 = from + 1, #tips2 do
		if tips2[i2].toShow then
			index2 = i2
			break
		end
	end
	if index2 ~= nil then
		return "show " .. index2 .. " " .. tips2[index2].id
	end
	return "none"
end
local tips3 = {{toShow = true, id = "a"}, {toShow = false, id = "b"}, {toShow = true, id = "c"}}
local v3 = print
local index3
for i3 = 1, #tips3 do
	if tips3[i3].toShow then
		index3 = i3
		break
	end
end
local v4 = if index3 == nil then "none" else "show " .. index3 .. " " .. tips3[index3].id
local index4
for i4 = 2, #tips3 do
	if tips3[i4].toShow then
		index4 = i4
		break
	end
end
local v5 = if index4 == nil then "none" else "show " .. index4 .. " " .. tips3[index4].id
local index5
for i5 = 4, #tips3 do
	if tips3[i5].toShow then
		index5 = i5
		break
	end
end
v3(v4, v5, if index5 == nil then "none" else "show " .. index5 .. " " .. tips3[index5].id)
local function findKey(t, want)
	local key
	for k, v6 in t do
		if v6 == want then
			key = k
			break
		end
	end
	if key ~= nil then
		return "found " .. key
	end
	return "none"
end
print(findKey({"x", "y", "z"}, "y"), (findKey({"x"}, "q")))
local function conv(list)
	local out = {}
	for _, v7 in list do
		local v8 = typeof(v7)
		v7 = if v8 == "number" then v7 else tonumber(v7) or -1
		table.insert(out, v7 * 2)
	end
	return table.concat(out, ",")
end
local function conv2(n)
	local out2 = {}
	for i6 = 1, n do
		local i7 = i6
		local v9 = i7 % 2
		i7 = if v9 == 0 then i7 * 100 else i7
		table.insert(out2, i7)
	end
	return table.concat(out2, ",")
end
print(conv({1, "2", "x", 4}), conv2(4))
local function m(items, opts)
	local res = {}
	for _2, it in items do
		if it.name then
			local n2 = it.name
			if opts and opts.skip and opts.skip[n2 .. "*"] then
				res[n2] = "skip*"
			elseif opts and opts.skip and opts.skip[n2] then
				res[n2] = "skip"
			else
				local k2 = it.kind
				if k2 == 1 then
					res[n2] = "one"
				elseif k2 == 2 then
					res[n2] = "two"
				elseif k2 == 3 then
					res[n2] = "three"
				end
			end
		elseif it.kind == 9 then
			res.schema = "s"
		end
	end
	local keys = {}
	for k3, v10 in res do
		table.insert(keys, k3 .. "=" .. v10)
	end
	table.sort(keys)
	return table.concat(keys, ",")
end
local items2 = {
	{name = "a", kind = 1},
	{name = "b", kind = 2},
	{name = "c", kind = 3},
	{name = "d", kind = 4},
	{kind = 9},
	{name = "e", kind = 1},
	{name = "f", kind = 1}
}
print(m(items2, nil))
print(m(items2, {skip = {["a*"] = true, b = true}}))
print(m(items2, {}))
