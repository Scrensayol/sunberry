local v1 = {util = {}}
local function area(v2, v3)
	return v2 * v3
end
v1.util.area = area
function v1.new(v4, v5, v6)
	return setmetatable({kind = v4, w = v5, h = v6}, {__index = v1})
end
function v1:describe()
	return self.kind .. " " .. v1.util.area(self.w, self.h)
end
local function parse(v7)
	local v8 = tonumber(v7)
	if v8 == nil then
		error("not a number: " .. v7)
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
for k, v12 in pairs(v10) do
	table.insert(v11, k .. "=" .. v12)
end
print(table.concat(v11))
local v13 = v9[3] and v9[3].kind or "none"
print(v13)
local v14 = if #v9 > 1 then "many" else "few"
print(v14)
