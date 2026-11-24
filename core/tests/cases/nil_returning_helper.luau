-- a helper that returns no values, inlined into a condition: its early `return` jumps to the
-- LOADNIL that gives the call its nil result, and its own tests sit among the condition's
local function hasName(names, name)
	local function inner()
		if type(names) == "table" then
		elseif type(names) == "string" then
		end
	end
	inner()
end
local function matches(schema, value)
	if schema.kind ~= value.kind or schema.name ~= nil and not hasName(schema.name, value.name) then
		return "no"
	end
	return "yes"
end
-- (through pcall, so the calls below aren't copies of `matches` too)
print(pcall(matches, { kind = 1, name = { "a" } }, { kind = 1 }))
print(pcall(matches, { kind = 1 }, { kind = 1 }))
print(pcall(matches, { kind = 1, name = "x" }, { kind = 2 }))
print(pcall(matches, { kind = 1, name = "x" }, { kind = 1 }))
