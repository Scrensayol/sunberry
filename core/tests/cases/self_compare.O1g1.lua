local function f(p1, p2)
	local v3 = p1
	if p2 then
		v3 = v3 == v3
	else
		v3 = "s"
	end
	print(v3)
	local v4 = p1
	local v5 = 1
	print(v5, v5 ~= v5, p1 < p1)
end
f(1, true)
f((0 / 0), true)
f("x", false)
