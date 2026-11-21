local v1 = {force = false, edge = true}
local v3 = {
	get = function(_, p2)
		return {fromEdge = p2 == "edge"}
	end
}
local v4 = {delivery = "https://delivery/", plain = "https://plain/"}
local function resolveHost()
	if v1.force then
		return v4.delivery
	end
	if v1.edge then
		local v5 = v3:get("edge")
		if v5 and v5.fromEdge then
			return v4.delivery
		end
	end
	return v4.plain
end
local function fetchIcons(p6)
	return string.format("%sicons?ids=%s", resolveHost(), table.concat(p6, ","))
end
print(fetchIcons({1, 2}))
v1.edge = false
print(fetchIcons({3}))
local function hasName(name, p7)
	if type(name) == "table" then
		for _, v in name do
			if v == p7 then
				return true
			end
		end
	elseif type(name) == "string" then
		return name == p7
	end
	return false
end
local function matches(p8, p9)
	if not (p8.kind == p9.kind and (p8.name == nil or hasName(p8.name, p9.name))) then
		return "no"
	end
	return "yes"
end
print(
	matches({kind = 1, name = {"a", "b"}}, {kind = 1, name = "b"}),
	matches({kind = 1, name = "x"}, {kind = 1, name = "y"})
)
local v10 = false
local function supported()
	if v10 then
		return v1.force
	end
	return true
end
local function enabled(p11, p12)
	return p11 and p12 and supported()
end
print(enabled(true, true), enabled(true, false))
v10 = true
print(enabled(true, true))
local function escape(name2)
	if name2 == "" then
		return "%00"
	end
	return (name2:gsub("%.", "%%2E"))
end
local function segmentFor(parent, p13)
	if parent.parent ~= nil then
		for _, v2 in p13 do
			if v2.node == parent then
				return v2.segment
			end
		end
	end
	return escape(parent.name)
end
local function pathOf(parent2, p14)
	local v15 = {}
	while parent2 do
		table.insert(v15, 1, segmentFor(parent2, p14))
		parent2 = parent2.parent
	end
	return table.concat(v15, "/")
end
local v16 = {name = ""}
local v17 = {name = "a.b", parent = v16}
local v18 = {name = "c", parent = v17}
print(pathOf(v18, {{segment = "M", node = v17}}), pathOf(v18, {}))
local v19 = {}
local v20 = 0
local function pick(p21)
	local v22 = if p21 > 2 then p21 * 10 else nil
	if v22 ~= nil then
		table.insert(v19, v22)
		return
	end
	if p21 < 0 then
		v20 += 1
	end
end
local function onInput(p23, p24, p25)
	if p23 == "click" then
		if not p25 then
			pick(p24)
		end
		return
	end
	if p23 == "touch" then
		v20 += 100
	end
end
onInput("click", 5, false)
onInput("click", -1, false)
onInput("click", 9, true)
onInput("touch", 0, false)
print(table.concat(v19, ","), v20)
local function walk(p26)
	local v27 = {}
	local v28 = 0
	local v29 = p26
	repeat
		v28 += 1
		local v30 = v29[v28]
		if not (v30 == nil or v30 == false) then
			table.insert(v27, v30)
		end
	until #v29 <= v28
	return table.concat(v27, ",")
end
print(walk({1, false, 3, 4}))
