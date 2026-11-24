local function hasName(names, name)
	if type(names) == "table" then
		for _, n in names do
			if n == name then
				return true
			end
		end
	elseif type(names) == "string" then
		return names == name
	end
	return false
end
local function matches(schema, value)
	if not (schema.kind == value.kind and (schema.name == nil or hasName(schema.name, value.name))) then
		return "no"
	end
	return "yes"
end
local v1 = print
local schema2 = {kind = 1, name = {"a", "b"}}
local value2 = {kind = 1, name = "b"}
local v2 = if schema2.kind == value2.kind and (schema2.name == nil or hasName(schema2.name, value2.name)) then "yes"
	else "no"
local schema3 = {kind = 1, name = "x"}
local value3 = {kind = 1, name = "y"}
v1(v2, if schema3.kind == value3.kind and (schema3.name == nil or hasName(schema3.name, value3.name)) then "yes"
	else "no")
local _
