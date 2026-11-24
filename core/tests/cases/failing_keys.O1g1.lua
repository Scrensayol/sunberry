local function show(...)
	print(...)
	return ...
end
local v1 = show(nil)
local success, result = pcall(function()
	local v2 = {x = 1, [v1] = show("value")}
	print("made", v2)
end)
print(success)
local v5 = pcall(function()
	local v3 = show("first")
	local v4 = {a = 1, [v1] = 2, b = show("second"), c = v3}
	print("made", v4)
end)
print(v5)
local v7 = pcall(function()
	local v6 = {a = 1, [nil] = 2}
	show("third")
	v6.b = 2
	print(v6)
end)
print(v7)
local v9 = pcall(function()
	local v8 = {[1] = type(true) == "boolean", [(0 / 0)] = 5}
	print("after", nil)
	v8[1] = 2
	print(v8)
end)
print(v9)
