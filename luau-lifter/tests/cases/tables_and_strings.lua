local function pack(...) -- name: pack
	return {
		["n"] = select("#", ...),
		...
	}
end
local v1 = {
	["volume"] = 0.8,
	["keybinds"] = {
		["jump"] = "Space",
		["sprint"] = "LeftShift"
	},
	["tags"] = { "fast", "quiet" },
	[10] = "ten"
}
local v7, v8 = (function(...) -- name: minMax
	-- upvalues: (copy) pack
	local v2 = pack(...)
	local huge = math.huge
	local v3 = -math.huge
	for v4 = 1, v2.n do
		local v5 = v2[v4]
		huge = math.min(huge, v5)
		local v6 = v2[v4]
		v3 = math.max(v3, v6)
	end
	return huge, v3
end)(4, 8, 15, 16, 23, 42)
local v9 = ("range %*..%*, volume %*%%"):format(v7, v8, v1.volume * 100)
print(v9, #v1.tags, v1.keybinds.jump, v1[10])
local v10 = {}
for v11, v12 in pairs(v1.keybinds) do
	v10[#v10 + 1] = v11 .. "=" .. v12
end
table.sort(v10)
print(table.concat(v10, ", "):upper())