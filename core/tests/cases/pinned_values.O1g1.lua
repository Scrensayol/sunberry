local function count(...)
	return select("#", ...)
end
g = 1.5
local v1 = {1, if g > 1 then g ^ 3 else 0, 9, count(1, 2)}
print(v1[2], #v1)
local v2 = {if g > 2 then 1.4142135623730951 else g ^ 2, count(1, 2, 3)}
print(v2[1], #v2)
local function run(p3, ...)
	local v4 = select("#", ...) ~= 10
	if p3 then
		for i = 2, 4 do
			v4 = -p3[i] > 65536
		end
	end
	return v4
end
print(run({1, -70000, -80000, 5}), run(nil, 1), run({1, 2, 3, -70000}))
