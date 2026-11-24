local function escape(s)
	if s ~= "" then
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
print(pathOf({name = "c", parent = mid}, {{segment = "M", node = mid}}))
local _
