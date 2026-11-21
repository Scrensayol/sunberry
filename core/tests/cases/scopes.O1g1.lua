local function scan(p1)
	local v2 = 0
	local v3 = {}
	local v4 = 1
	while v4 <= #p1 do
		local v5 = string.sub(p1, v4, v4)
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
	table.insert(v6, function()
		return v8
	end)
until v8 > 10
local v9 = {}
for _, v in v6 do
	table.insert(v9, v())
end
print(table.concat(v9, ","))
local function g(p10)
	local v11 = 0
	repeat
		v11 += 1
		local v12 = p10[v11]
	until v12 == nil or v12 < 0
	return v11
end
print(g({1, 2, -1, 5}), g({}))
