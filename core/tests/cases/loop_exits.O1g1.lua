local v1 = {}
local function v2()
	return {[2] = {seen = true}}
end
v1.get = v2
local function v5(v3)
	local v4 = v1.get()[v3]
	return if v4 == nil then false else v4.seen ~= nil
end
local function v8(v6)
	local v7
	for k, v in v6 do
		if v.show and not v5(v.id) then
			v7 = k
			break
		end
	end
	if v7 == nil then
		return false
	end
	local success, result = pcall(function()
		return v6[v7].id * 100
	end)
	return success, result
end
print(v8({
	{show = false, id = 1},
	{show = true, id = 2},
	{show = true, id = 3},
	{show = true, id = 4}
}))
print(v8({{show = true, id = 2}}))
local function v12(v9, v10)
	local v11
	for i = v10 + 1, #v9 do
		if v9[i].toShow then
			v11 = i
			break
		end
	end
	if v11 ~= nil then
		return "show " .. v11 .. " " .. v9[v11].id
	end
	return "none"
end
local v13 = {{toShow = true, id = "a"}, {toShow = false, id = "b"}, {toShow = true, id = "c"}}
print(v12(v13, 0), v12(v13, 1), v12(v13, 3))
local function v18(v14, v15)
	local v16
	for k2, v17 in v14 do
		if v17 == v15 then
			v16 = k2
			break
		end
	end
	if v16 ~= nil then
		return "found " .. v16
	end
	return "none"
end
print(v18({"x", "y", "z"}, "y"), v18({"x"}, "q"))
local function v23(v19)
	local v20 = {}
	for k3, v21 in v19 do
		local v22 = typeof(v21)
		table.insert(v20, (if v22 == "number" then v21 else tonumber(v21) or -1) * 2)
	end
	return table.concat(v20, ",")
end
local function v29(v24)
	local v25 = {}
	for i2 = 1, v24 do
		local v26 = i2
		local v27 = v26 % 2
		local v28 = if v27 == 0 then v26 * 100 else v26
		table.insert(v25, v28)
	end
	return table.concat(v25, ",")
end
print(v23({1, "2", "x", 4}), v29(4))
local function v36(v30, v31)
	local v32 = {}
	for k4, v33 in v30 do
		if v33.name then
			local name = v33.name
			if v31 and v31.skip and v31.skip[name .. "*"] then
				v32[name] = "skip*"
			elseif v31 and v31.skip and v31.skip[name] then
				v32[name] = "skip"
			else
				local kind = v33.kind
				if kind == 1 then
					v32[name] = "one"
				elseif kind == 2 then
					v32[name] = "two"
				elseif kind == 3 then
					v32[name] = "three"
				end
			end
		elseif v33.kind == 9 then
			v32.schema = "s"
		end
	end
	local v34 = {}
	for k5, v35 in v32 do
		table.insert(v34, k5 .. "=" .. v35)
	end
	table.sort(v34)
	return table.concat(v34, ",")
end
local v37 = {
	{name = "a", kind = 1},
	{name = "b", kind = 2},
	{name = "c", kind = 3},
	{name = "d", kind = 4},
	{kind = 9},
	{name = "e", kind = 1},
	{name = "f", kind = 1}
}
print(v36(v37, nil))
print(v36(v37, {skip = {["a*"] = true, b = true}}))
print(v36(v37, {}))
