-- tables, varargs, multiple returns and string building
local function pack(...)
	return { n = select("#", ...), ... }
end

local function minMax(...)
	local args = pack(...)
	local lo, hi = math.huge, -math.huge
	for i = 1, args.n do
		lo = math.min(lo, args[i])
		hi = math.max(hi, args[i])
	end
	return lo, hi
end

local settings = {
	volume = 0.8,
	keybinds = { jump = "Space", sprint = "LeftShift" },
	tags = { "fast", "quiet" },
	[10] = "ten",
}

local lo, hi = minMax(4, 8, 15, 16, 23, 42)
local summary = `range {lo}..{hi}, volume {settings.volume * 100}%`
print(summary, #settings.tags, settings.keybinds.jump, settings[10])

local parts = {}
for name, key in pairs(settings.keybinds) do
	parts[#parts + 1] = name .. "=" .. key
end
table.sort(parts)
print(table.concat(parts, ", "):upper())
