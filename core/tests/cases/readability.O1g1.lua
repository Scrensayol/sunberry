local v1 = {util = {}}
local function area(p2, p3)
	return p2 * p3
end
v1.util.area = area
function v1.new(kind, w, h)
	return setmetatable({kind = kind, w = w, h = h}, {__index = v1})
end
function v1:describe()
	return self.kind .. " " .. v1.util.area(self.w, self.h)
end
local function parse(p4)
	local v5 = tonumber(p4)
	if v5 == nil then
		error("not a number: " .. p4)
	end
	return v5
end
local success, result = pcall(parse, "12")
print(success, result)
local success2, result2 = pcall(parse, "twelve")
print(success2, (string.gsub(result2, "^.-: ", "")))
local v6 = {v1.new("square", 2, 2), v1.new("strip", 1, 9)}
for i, v in ipairs(v6) do
	print(i, v:describe())
end
local v7 = {a = 1}
local v8 = {}
for k, v2 in pairs(v7) do
	table.insert(v8, k .. "=" .. v2)
end
print(table.concat(v8))
local v9 = v6[3] and v6[3].kind or "none"
print(v9)
local v10 = if #v6 > 1 then "many" else "few"
print(v10)
