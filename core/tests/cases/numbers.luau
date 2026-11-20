-- constants the compiler folds, and the places folding differs from running
local base = 3
local exponent = 0.5
local half = 2 ^ exponent
local cube = base ^ 3
local square = base ^ 2
local huge = math.huge
local nan = 0 / 0

local function power(b, e)
	return b ^ e
end

print(half, cube, square, power(base, 3), power(2, exponent))
print(huge, -huge, nan ~= nan, 1 / -huge)
print(7 // 2, -7 // 2, 7 % -3, 2 ^ -1)
print(0x10, 1e3, 1.5e-3, 123456789012)

local acc = 1
for i = 1, 5 do
	acc *= i
	acc += 0.5
end
print(acc)

local message = `{base} cubed is {cube}, {#"four"} letters`
print(message)
