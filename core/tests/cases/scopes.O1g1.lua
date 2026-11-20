local function v6(v1)
	local v2 = 0
	local v3 = {}
	local v4 = 1
	while v4 <= #v1 do
		local v5 = string.sub(v1, v4, v4)
		if v5 == "(" then
			v2 = v2 + 1
		elseif v5 == ")" then
			v2 = v2 - 1
			if v2 < 0 then
				return "unbalanced at " .. v4
			end
		elseif v5 == "x" and v2 == 0 then
			table.insert(v3, v4)
		else
			v4 = v4 + 1
			continue
		end
		v4 = v4 + 1
	end
	if v2 ~= 0 then
		return "open"
	end
	return table.concat(v3, ",")
end
print(v6("x(x)x"), v6("(()"), v6("())x"), v6(""))
local v7 = {}
local v8 = 0
repeat
	v8 = v8 + 1
	local v9 = v8 * 3
	local function v10()
		return v9
	end
	table.insert(v7, v10)
until 10 < v9
local v11 = {}
for k, v in v7 do
	table.insert(v11, v())
end
print(table.concat(v11, ","))
local function v15(v12)
	local v13 = 0
	repeat
		v13 = v13 + 1
		local v14 = v12[v13]
	until v14 == nil or v14 < 0
	return v13
end
print(v15({1, 2, -1, 5}), v15({}))
