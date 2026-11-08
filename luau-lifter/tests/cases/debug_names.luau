-- compiled with debug level 2, so the real local names are in the bytecode
local function distance(ax, ay, bx, by)
	local dx = bx - ax
	local dy = by - ay
	return math.sqrt(dx * dx + dy * dy)
end

local waypoints = { { 0, 0 }, { 3, 4 }, { 6, 8 } }
local travelled = 0
for index = 2, #waypoints do
	local previous = waypoints[index - 1]
	local current = waypoints[index]
	travelled += distance(previous[1], previous[2], current[1], current[2])
end
print(travelled)
