-- a helper that returns nothing but calls another helper and drops its result, both inlined
-- into an argument list: the inner copy's early `return` jumps to the LOADNIL that is the outer
-- call's nil result, so the region is the outer helper (nil), not the inner one (0)
local zero = 0
local function bump()
	local function inner(t)
		if skip then
		else
			return zero
		end
	end
	inner(1)
end
print(bump(), bump())
for _ = 1, 1 do
	if skip then
	else
		print("loop", bump(), bump())
	end
end
