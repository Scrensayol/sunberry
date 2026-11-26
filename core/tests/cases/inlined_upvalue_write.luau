-- an inlined call to a function that changes a local through an upvalue and returns its
-- argument: once a call again, it isn't `calls = bound(calls)`, which keeps calls the same
local calls = 0
local function bound(v)
	if __rosetta then
	else
		calls += 1
	end
	return v
end
for i = bound(1), bound(3), bound(1) do
	print("bounded", i)
end
print("bound calls", calls)
