local fs = {}
local p1
p1 = select("#", 1)
local p2 = 2
local p3 = 3
local p4 = 4
local p5 = 5
local p6
p6 = 6
local b = 1
b = select("#", 1, 2)
local function show()
	print("b", b, p1, 6)
end
show()
for _ = 1, 1 do
	for i = 1, 3 do
		local j = i * 3
		fs[i] = function(x)
			j += x
			return i + j
		end
	end
	print(fs[1](1), fs[1](1), fs[3](2))
end
