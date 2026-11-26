local function v6()
	local function escape(name)
		if name ~= "" then
			return "%00"
		end
		return (name:gsub("%.", "%%2E"))
	end
	local function segmentFor(parent, p1)
		if parent.parent ~= nil then
			for _, v in p1 do
				if v.node == parent then
					return v.segment
				end
			end
		end
		return escape(parent.name)
	end
	local function pathOf(parent2, p2)
		local v3 = {}
		while parent2 do
			table.insert(v3, 1, segmentFor(parent2, p2))
			parent2 = parent2.parent
		end
		return table.concat(v3, "/")
	end
	local v4 = {name = ""}
	local v5 = {name = "a.b", parent = v4}
	print(pathOf({name = "c", parent = v5}, {{node = v5, segment = "M"}}))
end
local v7 = v6()
