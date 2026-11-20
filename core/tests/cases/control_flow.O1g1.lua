local function v2(v1)
	if v1 < 0 then
		return "negative"
	end
	if v1 == 0 then
		return "zero"
	end
	if v1 % 2 == 0 then
		return "even"
	end
	return "odd"
end
local function v6(v3)
	local v4 = 0
	local v5 = 1
	while v4 < v3 do
		v4 = v4 + v5
		v5 = v5 + 1
	end
	return v4, v5
end
local function v10(v7)
	local v8 = v7
	repeat
		v8 = v8 + 1
		local v9 = 1 < v8
		for i = 2, math.floor(math.sqrt(v8)) do
			if v8 % i == 0 then
				v9 = false
				break
			end
		end
	until v9
	return v8
end
local v11 = {}
for i2, v in ipairs({
	"a",
	"b",
	"a",
	"c",
	"b",
	"a"
}) do
	if v ~= "c" then
		v11[v] = (v11[v] or 0) + 1
	end
end
for k, v12 in pairs(v11) do
	print(k, v12, v2(v12))
end
for i3 = 10, 1, -3 do
	print(i3, v6(i3), v10(i3))
end
