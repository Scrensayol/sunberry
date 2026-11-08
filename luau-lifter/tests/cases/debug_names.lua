local waypoints = {
	{ 0, 0 },
	{ 3, 4 },
	{ 6, 8 }
}
local function distance(ax, ay, bx, by) -- name: distance
	local dx = bx - ax
	local dy = by - ay
	local v1 = dx * dx + dy * dy
	return math.sqrt(v1)
end
local travelled = 0
for index = 2, #waypoints do
	local previous = waypoints[index - 1]
	local current = waypoints[index]
	travelled = travelled + distance(previous[1], previous[2], current[1], current[2])
end
print(travelled)