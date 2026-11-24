-- an immediately called closure that makes the helpers is inlined, and its own body stays in
-- the chunk making them a second time: only the copy that runs counts as where they're made
local _ = (function()
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
	print(pathOf({ name = "c", parent = mid }, { { node = mid, segment = "M" } }))
end)()
