-- `a == a` reads one register twice. When `a` is the variable the expression assigns (the
-- branch's value goes back into it), both reads are that variable, with no temporary between
local function f(p, c)
	local a = p
	a = (if c then (a == a) else ((nil or 0) and "s"))
	print(a)
	local b = p
	b = (if false then (b == b) else 1)
	print(b, b ~= b, p < p)
end
f(1, true)
f(0 / 0, true)
f("x", false)
