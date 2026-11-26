local n = 1
local function bump()
	n += 10
	return n
end
local function show(f, _)
	print(tostring(n) .. tostring(f()), n)
	local t = {n, f(), n}
	print(t[1], t[3])
	print(n * f(), n)
end
show(bump, math.max)
