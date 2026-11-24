local v1 = {}
local function g(_, _)
end
local function f(p2)
	v1[4] = if rawequal(p2, nil) then tostring(g(g(1, math.max(2, 3)), 2) ~= 2)
		else `v=a{string.len("{braces}")}`
	return v1[4]
end
print(f(nil))
print(f(1))
local v3 = {h = 3, y = 16}
