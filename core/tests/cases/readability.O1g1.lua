local v1 = {util = {}}
local function v4(v2, v3)
	return v2 * v3
end
v1.util.area = v4
function v1.new(v5, v6, v7)
	return setmetatable({kind = v5, w = v6, h = v7}, {__index = v1})
end
function v1:describe()
	return self.kind .. " " .. v1.util.area(self.w, self.h)
end
local function v10(v8)
	local v9 = tonumber(v8)
	if v9 == nil then
		error("not a number: " .. v8)
	end
	return v9
end
local success, result = pcall(v10, "12")
print(success, result)
local success2, result2 = pcall(v10, "twelve")
print(success2, (string.gsub(result2, "^.-: ", "")))
local v11 = {v1.new("square", 2, 2), v1.new("strip", 1, 9)}
for i, v in ipairs(v11) do
	print(i, v:describe())
end
local v12 = {a = 1}
local v13 = {}
for k, v14 in pairs(v12) do
	table.insert(v13, k .. "=" .. v14)
end
print(table.concat(v13))
local v15 = v11[3] and v11[3].kind or "none"
print(v15)
local v16 = if #v11 > 1 then "many" else "few"
print(v16)
