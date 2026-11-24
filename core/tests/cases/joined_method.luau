-- an `if` expression whose then branch runs statements (what is left of inlined calls) is an
-- `if` statement with the value in one register on both sides; the else side calls a method on
-- that register, and the call's result still belongs to the same variable
local r = { h = 3, y = 16 }
local t = {}
local n, a = 2, "a"
local function g(p, q) end
local function f(c)
	t[4] = if rawequal(c, nil) then tostring(2 ~= g(g(1, math.max(n, r.h)), r.y ^ 0.25)) else `v={a}{string.len("{braces}")}`
	return t[4]
end
print(f(nil))
print(f(1))
