local function f(p, c)
	local a = p
	if c then
		a = a == a
	else
		a = "s"
	end
	print(a)
	local b = p
	b = 1
	print(b, b ~= b, p < p)
end
f(1, true)
f((0 / 0), true)
f("x")
