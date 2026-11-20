local function scan(v1)
	local v2 = 0
	local v3 = {}
	local v4 = 1
	while v4 <= #v1 do
		local v5 = string.sub(v1, v4, v4)
		if v5 == "(" then
			v2 += 1
		elseif v5 == ")" then
			v2 -= 1
			if v2 < 0 then
				return "unbalanced at " .. v4
			end
		elseif v5 == "x" and v2 == 0 then
			table.insert(v3, v4)
		else
			v4 += 1
			continue
		end
		v4 += 1
	end
	if v2 ~= 0 then
		return "open"
	end
	return table.concat(v3, ",")
end
print(scan("x(x)x"), scan("(()"), scan("())x"), scan(""))
local v6 = {}
local v7 = 0
repeat
	v7 += 1
	local v8 = v7 * 3
	local function v9()
		return v8
	end
	table.insert(v6, v9)
until 10 < v8
local v10 = {}
for _, v in v6 do
	table.insert(v10, v())
end
print(table.concat(v10, ","))
local function g(v11)
	local v12 = 0
	repeat
		v12 += 1
		local v13 = v11[v12]
	until v13 == nil or v13 < 0
	return v12
end
print(g({1, 2, -1, 5}), g({}))
