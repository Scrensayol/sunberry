local v1 = 0
local function bound(p2)
	if __rosetta then
		return p2
	end
	v1 += 1
	return p2
end
for i = bound(1), bound(3), bound(1) do
	print("bounded", i)
end
print("bound calls", v1)
