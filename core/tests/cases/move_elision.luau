-- Luau 0.746+ leaves an inlined call's result in the register its parameter was copied to, and
-- writes the last value of a multiple assignment straight into its target: at O2 debug level 2
-- a parameter's live range ends while its register is still read
local log = {}
local function tap(name, v)
	table.insert(log, name)
	return v
end
local t = {}
local a, b = tap("a", 1), tap("b", 2)
a, b = tap("b2", b), tap("a2", a)
t.x, t.y = tap("x", a), tap("y", b)
print(table.concat(log, ","), a, b, t.x, t.y)
