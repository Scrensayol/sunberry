local function pack(...)
	return {n = select("#", ...), ...}
end
local function minMax(...)
	local args = pack(...)
	local lo = (1 / 0)
	local hi = (-1 / 0)
	for i = 1, args.n do
		lo = math.min(lo, args[i])
		hi = math.max(hi, args[i])
	end
	return lo, hi
end
local settings = {
	volume = 0.8,
	keybinds = {jump = "Space", sprint = "LeftShift"},
	tags = {"fast", "quiet"},
	[10] = "ten"
}
local lo2, hi2 = minMax(4, 8, 15, 16, 23, 42)
local summary = `range {lo2}..{hi2}, volume {settings.volume * 100}%`
print(summary, #settings.tags, settings.keybinds.jump, settings[10])
local parts = {}
for name, key in pairs(settings.keybinds) do
	parts[#parts + 1] = name .. "=" .. key
end
table.sort(parts)
print(table.concat(parts, ", "):upper())
