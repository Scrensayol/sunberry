local log = {}
local function tap(tag, value)
	table.insert(log, tag)
	return value
end
local function clamp(x, lo, hi)
	if x < lo then
		return lo
	end
	if hi < x then
		return hi
	end
	return x
end
local function guard(value2)
	if not value2 then
		return
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
	print(i, (clamp(i, 0, 10)))
	bump(i)
end
local function scaled(n)
	return (clamp(n * 2, 1, 9))
end
if clamp(sum, 0, 5) == 5 and 6 > 4 then
	print("both")
end
while not (counter > 0) do
	counter -= clamp(counter, 1, 7)
	if counter % 3 == 0 then
		break
	end
end
print(sum, short, tbl.count, counter, table.concat(log, ","))
