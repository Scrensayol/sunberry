local function hasName(name, _)
	(function()
		if type(name) == "table" then
			return
		end
		if type(name) == "string" then
		end
	end)()
end
local function matches(p1, p2)
	if not (p1.kind == p2.kind and (p1.name == nil or hasName(p1.name, p2.name))) then
		return "no"
	end
	return "yes"
end
print(pcall(matches, {kind = 1, name = {"a"}}, {kind = 1}))
print(pcall(matches, {kind = 1}, {kind = 1}))
print(pcall(matches, {kind = 1, name = "x"}, {kind = 2}))
print(pcall(matches, {kind = 1, name = "x"}, {kind = 1}))
