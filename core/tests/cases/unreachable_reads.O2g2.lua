local function f(p)
	local n = math.clamp(p, -4, 4) * 2
	local m = (n + 0.5) % 3
	local w = 0
	while w < 2 and false do
		w += 1
		for k, v in {n, m, 4} do
			print(k, v)
		end
	end
	return w, m
end
print(f(1), f(9))
