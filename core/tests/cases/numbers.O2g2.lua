local base = 3
local exponent = 0.5
local half = 1.4142135623730951
local cube = 27
local square = 9
local huge = (1 / 0)
local nan = (0 / 0)
local function power(b, e)
	return b ^ e
end
print(1.4142135623730951, 27, 9, 27, 1.4142135623730951)
print((1 / 0), (-1 / 0), true, -0)
print(3, -4, -2, 0.5)
print(16, 1000, 0.0015, 123456789012)
local acc = 1
acc = acc * 1
acc = acc + 0.5
acc = acc * 2
acc = acc + 0.5
acc = acc * 3
acc = acc + 0.5
acc = acc * 4
acc = acc + 0.5
acc = acc * 5
acc = acc + 0.5
print(acc)
local message = ("%* cubed is %*, %* letters"):format(3, 27, 4)
print(message)
