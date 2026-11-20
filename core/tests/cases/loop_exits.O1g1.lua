local v1 = {}
local function get()
	return {[2] = {seen = true}}
end
v1.get = get
local function seen(p2)
	local v3 = v1.get()[p2]
	return if v3 == nil then false else v3.seen ~= nil
end
local function pick(p4)
	local v5
	for k, v in p4 do
		if v.show and not seen(v.id) then
			v5 = k
			break
		end
	end
	if v5 == nil then
		return false
	end
	local success, result = pcall(function()
		return p4[v5].id * 100
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
local function nextShown(p6, p7)
	local v8
	for i = p7 + 1, #p6 do
		if p6[i].toShow then
			v8 = i
			break
		end
	end
	if v8 ~= nil then
		return "show " .. v8 .. " " .. p6[v8].id
	end
	return "none"
end
local v9 = {{toShow = true, id = "a"}, {toShow = false, id = "b"}, {toShow = true, id = "c"}}
print(nextShown(v9, 0), nextShown(v9, 1), nextShown(v9, 3))
local function findKey(p10, p11)
	local v12
	for k2, v2 in p10 do
		if v2 == p11 then
			v12 = k2
			break
		end
	end
	if v12 ~= nil then
		return "found " .. v12
	end
	return "none"
end
print(findKey({"x", "y", "z"}, "y"), findKey({"x"}, "q"))
local function conv(p13)
	local v14 = {}
	for _, v4 in p13 do
		local v15 = typeof(v4)
		table.insert(v14, (if v15 == "number" then v4 else tonumber(v4) or -1) * 2)
	end
	return table.concat(v14, ",")
end
local function conv2(p16)
	local v17 = {}
	for i2 = 1, p16 do
		local v18 = i2
		local v19 = v18 % 2
		local v20 = if v19 == 0 then v18 * 100 else v18
		table.insert(v17, v20)
	end
	return table.concat(v17, ",")
end
print(conv({1, "2", "x", 4}), conv2(4))
local function m(p21, p22)
	local v23 = {}
	for _, v6 in p21 do
		if v6.name then
			local name = v6.name
			if p22 and p22.skip and p22.skip[name .. "*"] then
				v23[name] = "skip*"
			elseif p22 and p22.skip and p22.skip[name] then
				v23[name] = "skip"
			else
				local kind = v6.kind
				if kind == 1 then
					v23[name] = "one"
				elseif kind == 2 then
					v23[name] = "two"
				elseif kind == 3 then
					v23[name] = "three"
				end
			end
		elseif v6.kind == 9 then
			v23.schema = "s"
		end
	end
	local v24 = {}
	for k3, v7 in v23 do
		table.insert(v24, k3 .. "=" .. v7)
	end
	table.sort(v24)
	return table.concat(v24, ",")
end
local v25 = {
	{name = "a", kind = 1},
	{name = "b", kind = 2},
	{name = "c", kind = 3},
	{name = "d", kind = 4},
	{kind = 9},
	{name = "e", kind = 1},
	{name = "f", kind = 1}
}
print(m(v25, nil))
print(m(v25, {skip = {["a*"] = true, b = true}}))
print(m(v25, {}))
