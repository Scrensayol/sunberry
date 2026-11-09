local function scan(p1) -- name: scan
	local v2 = 1
	local v3 = 0
	local v4 = {}
	while true do
		local v5
		while true do
			if v2 > #p1 then
				return v3 ~= 0 and "open" or table.concat(v4, ",")
			end
			v5 = string.sub(p1, v2, v2)
			if v5 ~= "(" then
				break
			end
			v3 += 1
			v2 += 1
		end
		if v5 == ")" then
			v3 -= 1
			if v3 < 0 then
				return "unbalanced at " .. v2
			end
			v2 += 1
		elseif v5 == "x" and v3 == 0 then
			table.insert(v4, v2)
			v2 += 1
		else
			v2 += 1
		end
	end
end
print(scan("x(x)x"), scan("(()"), scan("())x"), scan(""))
local v6 = {}
local v7 = 0
repeat
	v7 += 1
	local v_u_8 = v7 * 3
	table.insert(v6, function()
		-- upvalues: (copy) v_u_8
		return v_u_8
	end)
until v_u_8 > 10
local v9 = {}
for _, v10 in v6 do
	table.insert(v9, v10())
end
print(table.concat(v9, ","))
local function g(p11) -- name: g
	local v12 = 0
	repeat
		v12 += 1
		local v13 = p11[v12]
	until v13 == nil or v13 < 0
	return v12
end
print(g({
	1,
	2,
	-1,
	5
}), g({}))