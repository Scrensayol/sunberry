-- the CLOSEUPVALS that ends `b`'s block comes right after an inlined call to a function that
-- captures it, so it ends up inside the copy that turns back into `show()`. After it, `b`'s
-- register is reused for `j`, and each closure has to get a `j` of its own
local fs = {}
do
	local p1, p2, p3, p4, p5, p6 = select("#", 1), 2, 3, 4, 5, 6
	local b = 1
	b = select("#", 1, 2)
	local function show()
		print("b", b, p1, p6)
	end
	show()
end
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
