local g = 0
local function bump()
	g += 1
	return g
end
if __rosetta then
	return
end
print(bump(), bump())
local _
