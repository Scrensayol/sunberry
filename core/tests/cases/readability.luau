-- shapes the printer turns back into source sugar
local Shapes = {}
Shapes.util = {}

function Shapes.util.area(w, h)
	return w * h
end

function Shapes.new(kind, w, h)
	return setmetatable({ kind = kind, w = w, h = h }, { __index = Shapes })
end

function Shapes:describe()
	return self.kind .. " " .. Shapes.util.area(self.w, self.h)
end

local function parse(text)
	local n = tonumber(text)
	if n == nil then
		error("not a number: " .. text)
	end
	return n
end

local success, result = pcall(parse, "12")
print(success, result)
local okay, err = pcall(parse, "twelve")
print(okay, (string.gsub(err, "^.-: ", "")))

local items = { Shapes.new("square", 2, 2), Shapes.new("strip", 1, 9) }
for i, shape in ipairs(items) do
	print(i, shape:describe())
end

local totals = { a = 1 }
local keys = {}
for key, value in pairs(totals) do
	table.insert(keys, key .. "=" .. value)
end
print(table.concat(keys))

local name = items[3] and items[3].kind or "none"
print(name)

local x
if #items > 1 then
	x = "many"
else
	x = "few"
end
print(x)
