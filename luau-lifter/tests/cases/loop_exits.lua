local v_u_2 = {
	get = function() -- name: get
		local v1 = {
			[2] = {
				seen = true
			}
		}
		return v1
	end
}
local function seen(p3) -- name: seen
	-- upvalues: (copy) v_u_2
	local v4 = v_u_2.get()[p3]
	local v5
	if v4 == nil then
		v5 = false
	else
		v5 = v4.seen ~= nil
	end
	return v5
end
local function pick(p_u_6) -- name: pick
	-- upvalues: (copy) seen
	local v_u_7 = nil
	for v8, v9 in p_u_6 do
		if v9.show and not seen(v9.id) then
			v_u_7 = v8
			break
		end
	end
	if v_u_7 == nil then
		return false
	end
	local v10, v11 = pcall(function()
		-- upvalues: (copy) p_u_6, (ref) v_u_7
		return p_u_6[v_u_7].id * 100
	end)
	return v10, v11
end
print(pick({
	{
		show = false,
		id = 1
	},
	{
		show = true,
		id = 2
	},
	{
		show = true,
		id = 3
	},
	{
		show = true,
		id = 4
	}
}))
print(pick({
	{
		show = true,
		id = 2
	}
}))
local function nextShown(p12, p13) -- name: nextShown
	local v14 = nil
	for v15 = p13 + 1, #p12 do
		if p12[v15].toShow then
			v14 = v15
			break
		end
	end
	return v14 == nil and "none" or "show " .. v14 .. " " .. p12[v14].id
end
local v16 = {
	{
		toShow = true,
		id = "a"
	},
	{
		toShow = false,
		id = "b"
	},
	{
		toShow = true,
		id = "c"
	}
}
print(nextShown(v16, 0), nextShown(v16, 1), nextShown(v16, 3))
local function findKey(p17, p18) -- name: findKey
	local v19 = nil
	for v20, v21 in p17 do
		if v21 == p18 then
			v19 = v20
			break
		end
	end
	return v19 == nil and "none" or "found " .. v19
end
print(findKey({ "x", "y", "z" }, "y"), findKey({ "x" }, "q"))
print((function(p22) -- name: conv
	local v23 = {}
	for _, v24 in p22 do
		local v25 = (typeof(v24) ~= "number" and (tonumber(v24) or -1) or v24) * 2
		table.insert(v23, v25)
	end
	return table.concat(v23, ",")
end)({
	1,
	"2",
	"x",
	4
}), (function(p26) -- name: conv2
	local v27 = {}
	for v28 = 1, p26 do
		local v29
		if v28 % 2 == 0 then
			v29 = v28 * 100
		else
			v29 = v28
		end
		table.insert(v27, v29)
	end
	return table.concat(v27, ",")
end)(4))
local function m(p30, p31) -- name: m
	local v32 = {}
	for _, v33 in p30 do
		if v33.name then
			local name = v33.name
			if p31 and (p31.skip and p31.skip[name .. "*"]) then
				v32[name] = "skip*"
			elseif p31 and (p31.skip and p31.skip[name]) then
				v32[name] = "skip"
			else
				local kind = v33.kind
				if kind == 1 then
					v32[name] = "one"
				elseif kind == 2 then
					v32[name] = "two"
				else
					if kind ~= 3 then
						continue
					end
					v32[name] = "three"
				end
			end
		else
			if v33.kind ~= 9 then
				continue
			end
			v32.schema = "s"
		end
	end
	local v34 = {}
	for v35, v36 in v32 do
		local v37 = v35 .. "=" .. v36
		table.insert(v34, v37)
	end
	table.sort(v34)
	return table.concat(v34, ",")
end
local v38 = {
	{
		name = "a",
		kind = 1
	},
	{
		name = "b",
		kind = 2
	},
	{
		name = "c",
		kind = 3
	},
	{
		name = "d",
		kind = 4
	},
	{
		kind = 9
	},
	{
		name = "e",
		kind = 1
	},
	{
		name = "f",
		kind = 1
	}
}
print(m(v38, nil))
local v39 = {
	skip = {
		["a*"] = true,
		b = true
	}
}
print(m(v38, v39))
print(m(v38, {}))