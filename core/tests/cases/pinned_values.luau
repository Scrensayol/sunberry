-- values decompilers pin so Luau can't fold them (`x ^ 3` is x*x*x at run time but pow()
-- when folded), inside an if-expression in a table constructor that ends in a call (an open
-- SETLIST, which can't be split into stores)
local function count(...)
	return select("#", ...)
end
g = 1.5
local t = { 1, (if g > 1 then g ^ 3 else 0), 9, count(1, 2) }
print(t[2], #t)
local u = { (if g > 2 then 2 ^ 0.5 else g ^ 2), count(1, 2, 3) }
print(u[1], #u)

-- an unrolled loop rewriting a boolean after an initializer that is itself a branch
local function run(r7, ...)
	local b1 = select("#", ...) ~= 10
	if r7 then
		for i = 2, 4 do
			b1 = (-r7[i]) > 65536
		end
	end
	return b1
end
print(run({ 1, -70000, -80000, 5 }), run(nil, 1), run({ 1, 2, 3, -70000 }))
