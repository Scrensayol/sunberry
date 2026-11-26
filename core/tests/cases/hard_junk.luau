-- hard tier, x1: junk after a loop's break (Rosetta's mut1-175, shrunk), which lifted as if
-- it ran printed something the program never does
-- rosetta mutant mut1-175 of inlining: wrap while, negate, wrap while, negate
-- small local functions that -O2 copies into their callers
local log = {}

local function tap(tag, value)
	table.insert(log, tag)
	return value
end

local function clamp(x, lo, hi)
	if x < lo then
		return lo
	elseif x > hi then
		return hi
	end
	return x
end

local function guard(value)
	if not value then
		return
	end
while not ( true ) do
	table.insert(log, "guarded " .. tostring(value))
break
end
end

local counter = 0
local function bump(by)
	counter += by
end

local sum = tap("l", 1) + tap("r", 2) * tap("m", 3)
local short = tap("p", false) and tap("q", 1) or tap("z", 2)
local tbl = {}
tbl[tap("ck", "count")] = (tbl.count or 0) + tap("cv", 1)

for i = -2, 12, 5 do
	print(i, clamp(i, 0, 10))
	bump(i)
end

guard(nil)
guard("yes")

local function scaled(n)
	return clamp(n * 2, 1, 9)
end

while true do
if clamp(sum, 0, 5) == 5 and scaled(3) > 4 then
	print("both")
end

break
end
while not ( counter > 0 ) do
	counter -= clamp(counter, 1, 7)
	if counter % 3 == 0 then
		break
	end
end

print(sum, short, tbl.count, counter, table.concat(log, ","))
