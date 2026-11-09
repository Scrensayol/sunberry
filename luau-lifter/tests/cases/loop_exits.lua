local v_u_1 = {
	get = function() -- name: get
		return {
			[2] = {
				seen = true
			}
		}
	end
}
local function seen(p2) -- name: seen
	-- upvalues: (copy) v_u_1
	local v3 = v_u_1.get()[p2]
	local v4
	if v3 == nil then
		v4 = false
	else
		v4 = v3.seen ~= nil
	end
	return v4
end
local function pick(p_u_5) -- name: pick
	-- upvalues: (copy) seen
	local v_u_6 = nil
	for v7, v8 in p_u_5 do
		if v8.show and not seen(v8.id) then
			v_u_6 = v7
			break
		end
	end
	if v_u_6 == nil then
		return false
	end
	local v9, v10 = pcall(function()
		-- upvalues: (copy) p_u_5, (ref) v_u_6
		return p_u_5[v_u_6].id * 100
	end)
	return v9, v10
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
local function nextShown(p11, p12) -- name: nextShown
	local v13 = nil
	for v14 = p12 + 1, #p11 do
		if p11[v14].toShow then
			v13 = v14
			break
		end
	end
	return v13 == nil and "none" or "show " .. v13 .. " " .. p11[v13].id
end
local v15 = {
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
print(nextShown(v15, 0), nextShown(v15, 1), nextShown(v15, 3))
local function findKey(p16, p17) -- name: findKey
	local v18 = nil
	for v19, v20 in p16 do
		if v20 == p17 then
			v18 = v19
			break
		end
	end
	return v18 == nil and "none" or "found " .. v18
end
print(findKey({ "x", "y", "z" }, "y"), findKey({ "x" }, "q"))
print((function(p21) -- name: conv
	local v22 = {}
	for _, v23 in p21 do
		local v24 = (typeof(v23) ~= "number" and (tonumber(v23) or -1) or v23) * 2
		table.insert(v22, v24)
	end
	return table.concat(v22, ",")
end)({
	1,
	"2",
	"x",
	4
}), (function(p25) -- name: conv2
	local v26 = {}
	for v27 = 1, p25 do
		local v28
		if v27 % 2 == 0 then
			v28 = v27 * 100
		else
			v28 = v27
		end
		table.insert(v26, v28)
	end
	return table.concat(v26, ",")
end)(4))
local function m(p29, p30) -- name: m
	local v31 = {}
	for _, v32 in p29 do
		if v32.name then
			local name = v32.name
			if p30 and (p30.skip and p30.skip[name .. "*"]) then
				v31[name] = "skip*"
			elseif p30 and (p30.skip and p30.skip[name]) then
				v31[name] = "skip"
			else
				local kind = v32.kind
				if kind == 1 then
					v31[name] = "one"
				elseif kind == 2 then
					v31[name] = "two"
				else
					if kind ~= 3 then
						continue
					end
					v31[name] = "three"
				end
			end
		else
			if v32.kind ~= 9 then
				continue
			end
			v31.schema = "s"
		end
	end
	local v33 = {}
	for v34, v35 in v31 do
		local v36 = v34 .. "=" .. v35
		table.insert(v33, v36)
	end
	table.sort(v33)
	return table.concat(v33, ",")
end
local v37 = {
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
print(m(v37, nil))
print(m(v37, {
	skip = {
		["a*"] = true,
		b = true
	}
}))
print(m(v37, {}))