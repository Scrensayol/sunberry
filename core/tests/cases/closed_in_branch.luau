-- a branch that returns closes the upvalues it shares on the way out (CLOSEUPVALS), which says
-- nothing about the code after the `if`: there `g` is still shared with `bump`, so the copy of
-- bump that -O2 inlines into the call has to assign it, or the real call after it reads 0
local g = 0
local function bump()
	g += 1
	for _ = 1, 1 do
		return g
	end
end
if __rosetta then
	return
end
local _ = (function()
	print(bump(), bump())
end)()
