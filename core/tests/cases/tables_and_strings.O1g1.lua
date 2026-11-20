local function v1(...)
	return {n = select("#", ...), ...}
end
local function v4(...)
	local v2 = v1(...)
	local huge = math.huge
	local v3 = -math.huge
	for i = 1, v2.n do
		huge = math.min(huge, v2[i])
		v3 = math.max(v3, v2[i])
	end
	return huge, v3
end
local v5 = {
	volume = 0.8,
	keybinds = {jump = "Space", sprint = "LeftShift"},
	tags = {"fast", "quiet"},
	[10] = "ten"
}
local v6, v7 = v4(4, 8, 15, 16, 23, 42)
local v8 = ("range %*..%*, volume %*%%"):format(v6, v7, v5.volume * 100)
print(v8, #v5.tags, v5.keybinds.jump, v5[10])
local v9 = {}
for k, v in pairs(v5.keybinds) do
	v9[#v9 + 1] = k .. "=" .. v
end
table.sort(v9)
print(table.concat(v9, ", "):upper())
