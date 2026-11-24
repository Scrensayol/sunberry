local function f(p1)
	local v2 = math.clamp(p1, -4, 4) * 2
	local v3 = (v2 + 0.5) % 3
	local v4 = 0
	while v4 < 2 and false do
		v4 += 1
		for k, v in {v2, v3, 4} do
			print(k, v)
		end
	end
	return v4, v3
end
print(f(1), f(9))
