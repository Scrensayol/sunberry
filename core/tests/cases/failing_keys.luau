-- a table with a nil or NaN key is an error where it is made, so its constructor can't wait
-- for a later statement the way one that can't fail does
local function show(...) print(...) return ... end
local k = show(nil)
local ok, e = pcall(function()
	local t = {x = 1, [k] = show("value")}
	print("made", t)
end)
print(ok)
local ok2 = pcall(function()
	local v = show("first")
	local t = {a = 1, [k] = 2, b = show("second"), c = v}
	print("made", t)
end)
print(ok2)
local ok3 = pcall(function()
	local t = {a = 1, [nil] = 2}
	show("third")
	t.b = 2
	print(t)
end)
print(ok3)
local ok4 = pcall(function()
	local a
	local t = {[1] = (type(true) == "boolean"), [(0/0)] = 5}
	print("after", a)
	t[1] = 2
	print(t)
end)
print(ok4)
