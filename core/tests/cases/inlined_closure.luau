-- an immediately called closure that -O2 inlines stays in the chunk with nothing creating it:
-- what it can't lift on its own doesn't matter, since it is never printed
local function hasName(names, name)
	if type(names) == "table" then
		for _, n in names do
			if n == name then
				return true
			end
		end
	elseif type(names) == "string" then
		for _ = 1, 2 do
			return names == name
		end
	end
	return false
end
local function matches(schema, value)
	if schema.kind ~= value.kind or schema.name ~= nil and not hasName(schema.name, value.name) then
		return "no"
	end
	return "yes"
end
local _ = (function()
print(matches({ kind = 1, name = { "a", "b" } }, { kind = 1, name = "b" }), matches({ kind = 1, name = "x" }, { kind = 1, name = "y" }))
end)()
