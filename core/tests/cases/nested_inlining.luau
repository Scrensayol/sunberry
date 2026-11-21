-- shapes from inlined helpers with control flow (-O2 copies them into their callers)

local flags = { force = false, edge = true }
local layers = { get = function(_, key) return { fromEdge = key == "edge" } end }
local urls = {}
urls.delivery = "https://delivery/"
urls.plain = "https://plain/"

-- early returns, one inside a condition with a call in it, inlined into an argument
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
print(fetchIcons({ 1, 2 }))
flags.edge = false
print(fetchIcons({ 3 }))

-- a return from inside a loop
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
	if schema.kind ~= value.kind or schema.name ~= nil and not hasName(schema.name, value.name) then
		return "no"
	end
	return "yes"
end
print(matches({ kind = 1, name = { "a", "b" } }, { kind = 1, name = "b" }), matches({ kind = 1, name = "x" }, { kind = 1, name = "y" }))

-- `return a and b and f()`, with f returning early
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
print(enabled(true, true), enabled(true, false))
vr = true
print(enabled(true, true))

-- a helper inlined into one that is inlined in turn, returning from a loop
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
	return escape(node.name)
end
local function pathOf(node, known)
	local parts = {}
	while node do
		table.insert(parts, 1, segmentFor(node, known))
		node = node.parent
	end
	return table.concat(parts, "/")
end
local root = { name = "" }
local mid = { name = "a.b", parent = root }
local leaf = { name = "c", parent = mid }
print(pathOf(leaf, { { node = mid, segment = "M" } }), pathOf(leaf, {}))

-- a helper whose returns all lead to its caller's end
local picks, misses = {}, 0
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
local function onInput(kind, x, blocked)
	if kind == "click" then
		if not blocked then
			pick(x)
		end
	elseif kind == "touch" then
		misses += 100
	end
end
onInput("click", 5, false)
onInput("click", -1, false)
onInput("click", 9, true)
onInput("touch", 0, false)
print(table.concat(picks, ","), misses)

-- `continue` in a `repeat` goes to its condition
local function walk(list)
	local out, i, stack = {}, 0, list
	repeat
		i += 1
		local v = stack[i]
		if v == nil or v == false then
			continue
		end
		table.insert(out, v)
	until i >= #stack
	return table.concat(out, ",")
end
print(walk({ 1, false, 3, 4 }))
