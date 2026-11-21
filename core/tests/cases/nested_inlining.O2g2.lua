local layers = {
	get = function(_, key)
		return {fromEdge = key == "edge"}
	end
}
local urls = {}
urls.delivery = "https://delivery/"
urls.plain = "https://plain/"
local flags = {force = false, edge = true}
local function resolveHost()
	if flags.force then
		return urls.delivery
	end
	if flags.edge then
		local data = layers:get("edge")
		if data and data.fromEdge then
			return urls.delivery
		end
	end
	return urls.plain
end
local function fetchIcons(ids)
	return string.format("%sicons?ids=%s", resolveHost(), table.concat(ids, ","))
end
print(fetchIcons({1, 2}))
flags.edge = false
print(fetchIcons({3}))
local function hasName(names, name)
	if type(names) == "table" then
		for _, n in names do
			if n == name then
				return true
			end
		end
	elseif type(names) == "string" then
		return names == name
	end
	return false
end
local function matches(schema, value)
	if not (schema.kind == value.kind and (schema.name == nil or hasName(schema.name, value.name))) then
		return "no"
	end
	return "yes"
end
local v1 = print
local schema2 = {kind = 1, name = {"a", "b"}}
local value2 = {kind = 1, name = "b"}
local v2 = if schema2.kind == value2.kind and (schema2.name == nil or hasName(schema2.name, value2.name)) then "yes"
	else "no"
local schema3 = {kind = 1, name = "x"}
local value3 = {kind = 1, name = "y"}
v1(v2, if schema3.kind == value3.kind and (schema3.name == nil or hasName(schema3.name, value3.name)) then "yes"
	else "no")
local vr = false
local function supported()
	if vr then
		return flags.force
	end
	return true
end
local function enabled(a, b)
	return a and b and supported()
end
print(supported(), false)
vr = true
print((supported()))
local function escape(s)
	if s == "" then
		return "%00"
	end
	return (s:gsub("%.", "%%2E"))
end
local function segmentFor(node, known)
	if node.parent ~= nil then
		for _, entry in known do
			if entry.node == node then
				return entry.segment
			end
		end
	end
	return (escape(node.name))
end
local function pathOf(node2, known2)
	local parts = {}
	while node2 do
		table.insert(parts, 1, (segmentFor(node2, known2)))
		node2 = node2.parent
	end
	return table.concat(parts, "/")
end
local root = {name = ""}
local mid = {name = "a.b", parent = root}
local leaf = {name = "c", parent = mid}
print(pathOf(leaf, {{segment = "M", node = mid}}), pathOf(leaf, {}))
local misses = 0
local picks = {}
local function pick(x)
	local hit = if x > 2 then x * 10 else nil
	if hit ~= nil then
		table.insert(picks, hit)
		return
	end
	if x < 0 then
		misses += 1
	end
end
local function onInput(kind, x2, blocked)
	if kind == "click" then
		if not blocked then
			pick(x2)
		end
		return
	end
	if kind == "touch" then
		misses += 100
	end
end
local hit2 = 50
table.insert(picks, 50)
local hit3
misses += 1
misses += 100
print(table.concat(picks, ","), misses)
local function walk(list)
	local i = 0
	local stack = list
	local out = {}
	repeat
		i += 1
		local v = stack[i]
		if not (v == nil or v == false) then
			table.insert(out, v)
		end
	until #stack <= i
	return table.concat(out, ",")
end
print(walk({1, false, 3, 4}))
