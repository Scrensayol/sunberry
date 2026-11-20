local v1 = {util = {}}
local function area(p2, p3)
	return p2 * p3
end
v1.util.area = area
function v1.new(p4, p5, p6)
	return setmetatable({kind = p4, w = p5, h = p6}, {__index = v1})
end
function v1:describe()
	return self.kind .. " " .. v1.util.area(self.w, self.h)
end
local function parse(p7)
	local v8 = tonumber(p7)
	if v8 == nil then
		error("not a number: " .. p7)
	end
	return v8
end
local success, result = pcall(parse, "12")
print(success, result)
local success2, result2 = pcall(parse, "twelve")
print(success2, (string.gsub(result2, "^.-: ", "")))
local v9 = {v1.new("square", 2, 2), v1.new("strip", 1, 9)}
for i, v in ipairs(v9) do
	print(i, v:describe())
end
local v10 = {a = 1}
local v11 = {}
for k, v2 in pairs(v10) do
	table.insert(v11, k .. "=" .. v2)
end
print(table.concat(v11))
local v12 = v9[3] and v9[3].kind or "none"
print(v12)
local v13 = if #v9 > 1 then "many" else "few"
print(v13)
