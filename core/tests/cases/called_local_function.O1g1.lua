local v1 = 1
local function bump()
	v1 += 10
	return v1
end
local function show(callback, _)
	print(tostring(v1) .. tostring(callback()), v1)
	local v2 = {v1, callback(), v1}
	print(v2[1], v2[3])
	print(v1 * callback(), v1)
end
show(bump, math.max)
