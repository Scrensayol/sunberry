local huge = math.huge
local function power(p1, p2)
	return p1 ^ p2
end
print(1.4142135623730951, 27, 9, power(3, 3), power(2, 0.5))
print(huge, -huge, true, 1 / -huge)
print(3, -4, -2, 0.5)
print(16, 1000, 0.0015, 123456789012)
local v3 = 1
for i = 1, 5 do
	v3 *= i
	v3 += 0.5
end
print(v3)
local v4 = `{3} cubed is {27}, {4} letters`
print(v4)
