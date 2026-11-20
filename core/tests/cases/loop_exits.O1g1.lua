local v1 = {}
local function get()
	return {[2] = {seen = true}}
end
v1.get = get
local function seen(v2)
	local v3 = v1.get()[v2]
	return if v3 == nil then false else v3.seen ~= nil
end
local function pick(v4)
	local v5
	for k, v in v4 do
		if v.show and not seen(v.id) then
			v5 = k
			break
		end
	end
	if v5 == nil then
		return false
	end
	local success, result = pcall(function()
		return v4[v5].id * 100
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
local function nextShown(v6, v7)
	local v8
	for i = v7 + 1, #v6 do
		if v6[i].toShow then
			v8 = i
			break
		end
	end
	if v8 ~= nil then
		return "show " .. v8 .. " " .. v6[v8].id
	end
	return "none"
end
local v9 = {{toShow = true, id = "a"}, {toShow = false, id = "b"}, {toShow = true, id = "c"}}
print(nextShown(v9, 0), nextShown(v9, 1), nextShown(v9, 3))
local function findKey(v10, v11)
	local v12
	for k2, v13 in v10 do
		if v13 == v11 then
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
local function conv(v14)
	local v15 = {}
	for _, v16 in v14 do
		local v17 = typeof(v16)
		table.insert(v15, (if v17 == "number" then v16 else tonumber(v16) or -1) * 2)
	end
	return table.concat(v15, ",")
end
local function conv2(v18)
	local v19 = {}
	for i2 = 1, v18 do
		local v20 = i2
		local v21 = v20 % 2
		local v22 = if v21 == 0 then v20 * 100 else v20
		table.insert(v19, v22)
	end
	return table.concat(v19, ",")
end
print(conv({1, "2", "x", 4}), conv2(4))
local function m(v23, v24)
	local v25 = {}
	for _, v26 in v23 do
		if v26.name then
			local name = v26.name
			if v24 and v24.skip and v24.skip[name .. "*"] then
				v25[name] = "skip*"
			elseif v24 and v24.skip and v24.skip[name] then
				v25[name] = "skip"
			else
				local kind = v26.kind
				if kind == 1 then
					v25[name] = "one"
				elseif kind == 2 then
					v25[name] = "two"
				elseif kind == 3 then
					v25[name] = "three"
				end
			end
		elseif v26.kind == 9 then
			v25.schema = "s"
		end
	end
	local v27 = {}
	for k3, v28 in v25 do
		table.insert(v27, k3 .. "=" .. v28)
	end
	table.sort(v27)
	return table.concat(v27, ",")
end
local v29 = {
	{name = "a", kind = 1},
	{name = "b", kind = 2},
	{name = "c", kind = 3},
	{name = "d", kind = 4},
	{kind = 9},
	{name = "e", kind = 1},
	{name = "f", kind = 1}
}
print(m(v29, nil))
print(m(v29, {skip = {["a*"] = true, b = true}}))
print(m(v29, {}))
