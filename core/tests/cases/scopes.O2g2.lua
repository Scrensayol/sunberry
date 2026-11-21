local function scan(s)
	local depth = 0
	local out = {}
	local i = 1
	while i <= #s do
		local c = string.sub(s, i, i)
		if c == "(" then
			depth += 1
		elseif c == ")" then
			depth -= 1
			if depth < 0 then
				return "unbalanced at " .. i
			end
		elseif c == "x" and depth == 0 then
			table.insert(out, i)
		else
			i += 1
			continue
		end
		i += 1
	end
	if depth ~= 0 then
		return "open"
	end
	return table.concat(out, ",")
end
print(scan("x(x)x"), scan("(()"), scan("())x"), scan(""))
local fs = {}
local i2 = 0
repeat
	i2 += 1
	local x = i2 * 3
	table.insert(fs, function()
		return x
	end)
until x > 10
local r = {}
for _, f in fs do
	table.insert(r, f())
end
print(table.concat(r, ","))
local function g(t)
	local n = 0
	repeat
		n += 1
		local v = t[n]
	until v == nil or v < 0
	return n
end
print(g({1, 2, -1, 5}), (g({})))
