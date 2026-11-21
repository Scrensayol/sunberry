local v1 = {
	get = function()
		return {[2] = {seen = true}}
	end
}
local function seen(id)
	local v2 = v1.get()[id]
	return if v2 == nil then false else v2.seen ~= nil
end
local function pick(p3)
	local v4
	for k, v in p3 do
		if v.show and not seen(v.id) then
			v4 = k
			break
		end
	end
	if v4 == nil then
		return false
	end
	local success, result = pcall(function()
		return p3[v4].id * 100
	end)
	return success, result
end
print(pick({
	{show = false, id = 1},
	{show = true, id = 2},
	{show = true, id = 3},
	{show = true, id = 4}
}))
print(pick({{show = true, id = 2}}))
local function nextShown(p5, p6)
	local v7
	for i = p6 + 1, #p5 do
		if p5[i].toShow then
			v7 = i
			break
		end
	end
	if v7 ~= nil then
		return "show " .. v7 .. " " .. p5[v7].id
	end
	return "none"
end
local v8 = {{toShow = true, id = "a"}, {toShow = false, id = "b"}, {toShow = true, id = "c"}}
print(nextShown(v8, 0), nextShown(v8, 1), nextShown(v8, 3))
local function findKey(p9, p10)
	local v11
	for k2, v3 in p9 do
		if v3 == p10 then
			v11 = k2
			break
		end
	end
	if v11 ~= nil then
		return "found " .. v11
	end
	return "none"
end
print(findKey({"x", "y", "z"}, "y"), findKey({"x"}, "q"))
local function conv(p12)
	local v13 = {}
	for _, v5 in p12 do
		local v14 = typeof(v5)
		table.insert(v13, (if v14 == "number" then v5 else tonumber(v5) or -1) * 2)
	end
	return table.concat(v13, ",")
end
local function conv2(p15)
	local v16 = {}
	for i2 = 1, p15 do
		local v17 = i2
		local v18 = v17 % 2
		local v19 = if v18 == 0 then v17 * 100 else v17
		table.insert(v16, v19)
	end
	return table.concat(v16, ",")
end
print(conv({1, "2", "x", 4}), conv2(4))
local function m(p20, p21)
	local v22 = {}
	for _, v6 in p20 do
		if v6.name then
			local name = v6.name
			if p21 and p21.skip and p21.skip[name .. "*"] then
				v22[name] = "skip*"
			elseif p21 and p21.skip and p21.skip[name] then
				v22[name] = "skip"
			else
				local kind = v6.kind
				if kind == 1 then
					v22[name] = "one"
				elseif kind == 2 then
					v22[name] = "two"
				elseif kind == 3 then
					v22[name] = "three"
				end
			end
		elseif v6.kind == 9 then
			v22.schema = "s"
		end
	end
	local v23 = {}
	for k3, v9 in v22 do
		table.insert(v23, k3 .. "=" .. v9)
	end
	table.sort(v23)
	return table.concat(v23, ",")
end
local v24 = {
	{name = "a", kind = 1},
	{name = "b", kind = 2},
	{name = "c", kind = 3},
	{name = "d", kind = 4},
	{kind = 9},
	{name = "e", kind = 1},
	{name = "f", kind = 1}
}
print(m(v24, nil))
print(m(v24, {skip = {["a*"] = true, b = true}}))
print(m(v24, {}))
