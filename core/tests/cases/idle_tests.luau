-- `x and 0/0 or 0/0` is always true, so a test of it goes to the same place either way and the
-- `else nil` arm of the `if` around it is never reached: code that does nothing, in the way of
-- the condition it sits in
local t = { flag = false }
local function f(a, n)
	local i = 0
	repeat
		i += 1
		if not ((a or true) and -1) then
			print("never")
		elseif (t.flag and false or a) or (if (t.flag and (0 / 0) or (0 / 0)) then "s" else nil) then
			t.flag = type(a) == "string"
			print("set", t.flag)
		end
	until i >= n or true
	return i
end
print(f(nil, 3), f("x", 1), t.flag)
