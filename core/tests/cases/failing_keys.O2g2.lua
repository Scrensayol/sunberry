local function show(...)
	print(...)
	return ...
end
local k = show(nil)
local ok, e = pcall(function()
	local t = {x = 1, [k] = show("value")}
	print("made", t)
end)
print(ok)
local ok2 = pcall(function()
	local v = show("first")
	local t2 = {a = 1, [k] = 2, b = show("second"), c = v}
	print("made", t2)
end)
print(ok2)
local ok3 = pcall(function()
	local t3 = {a = 1, [nil] = 2}
	show("third")
	t3.b = 2
	print(t3)
end)
print(ok3)
local ok4 = pcall(function()
	local a
	local t4 = {[1] = true, [(0 / 0)] = 5}
	print("after", nil)
	t4[1] = 2
	print(t4)
end)
print(ok4)
