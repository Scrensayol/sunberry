local function count(...)
	return select("#", ...)
end
g = 1.5
local t = {1, if g > 1 then g ^ 3 else 0, 9, count(1, 2)}
print(t[2], #t)
local u = {if g > 2 then 1.4142135623730951 else g ^ 2, count(1, 2, 3)}
print(u[1], #u)
local function run(r7, ...)
	local b1 = select("#", ...) ~= 10
	if r7 then
		b1 = -r7[2] > 65536
		b1 = -r7[3] > 65536
		b1 = -r7[4] > 65536
	end
	return b1
end
print(run({1, -70000, -80000, 5}), run(nil, 1), run({1, 2, 3, -70000}))
