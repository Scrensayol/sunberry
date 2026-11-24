local function hasName(names, _)
	local function inner()
		if type(names) == "table" then
			return
		end
		if type(names) == "string" then
		end
	end
	inner()
end
local function matches(schema, value)
	local v1 = schema.kind == value.kind
	if v1 then
		local v2 = schema.name == nil
		if not v2 then
			local names2 = schema.name
			local name = value.name
			v2 = hasName(names2)
		end
		v1 = v2
	end
	if not v1 then
		return "no"
	end
	return "yes"
end
print(pcall(matches, {kind = 1, name = {"a"}}, {kind = 1}))
print(pcall(matches, {kind = 1}, {kind = 1}))
print(pcall(matches, {kind = 1, name = "x"}, {kind = 2}))
print(pcall(matches, {kind = 1, name = "x"}, {kind = 1}))
