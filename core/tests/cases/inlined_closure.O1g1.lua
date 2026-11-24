local function hasName(name, p1)
	if type(name) == "table" then
		for _, v in name do
			if v == p1 then
				return true
			end
		end
	elseif type(name) == "string" then
		for _ = 1, 2 do
			return name == p1
		end
	end
	return false
end
local function matches(p2, p3)
	if not (p2.kind == p3.kind and (p2.name == nil or hasName(p2.name, p3.name))) then
		return "no"
	end
	return "yes"
end
local v4 = (function()
	print(
		matches({kind = 1, name = {"a", "b"}}, {kind = 1, name = "b"}),
		matches({kind = 1, name = "x"}, {kind = 1, name = "y"})
	)
end)()
