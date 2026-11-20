local function pack(...)
	return {n = select("#", ...), ...}
end
local function minMax(...)
	local v1 = pack(...)
	local huge = math.huge
	local v2 = -math.huge
	for i = 1, v1.n do
		huge = math.min(huge, v1[i])
		v2 = math.max(v2, v1[i])
	end
	return huge, v2
end
local v3 = {
	volume = 0.8,
	keybinds = {jump = "Space", sprint = "LeftShift"},
	tags = {"fast", "quiet"},
	[10] = "ten"
}
local v4, v5 = minMax(4, 8, 15, 16, 23, 42)
local v6 = `range {v4}..{v5}, volume {v3.volume * 100}%`
print(v6, #v3.tags, v3.keybinds.jump, v3[10])
local v7 = {}
for k, v in pairs(v3.keybinds) do
	v7[#v7 + 1] = k .. "=" .. v
end
table.sort(v7)
print(table.concat(v7, ", "):upper())
