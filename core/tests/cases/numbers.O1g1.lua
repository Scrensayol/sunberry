local huge = math.huge
local function v3(v1, v2)
	return v1 ^ v2
end
print(1.4142135623730951, 27, 9, v3(3, 3), v3(2, 0.5))
print(huge, -huge, true, 1 / -huge)
print(3, -4, -2, 0.5)
print(16, 1000, 0.0015, 123456789012)
local v4 = 1
for i = 1, 5 do
	v4 = v4 * i
	v4 = v4 + 0.5
end
print(v4)
local v5 = ("%* cubed is %*, %* letters"):format(3, 27, 4)
print(v5)
