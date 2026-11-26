local v1 = 0
local function bump()
	v1 += 1
	for _ = 1, 1 do
		return v1
	end
end
if __rosetta then
	return
end
local function v2()
	print(bump(), bump())
end
local v3 = v2()
