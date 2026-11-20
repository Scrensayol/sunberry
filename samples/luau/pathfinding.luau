-- A* on a small grid map, with a binary heap for the open set.

local MAP = {
	"S....#....",
	".##..#.##.",
	".#...#..#.",
	".#.###..#.",
	"...#....#G",
	"##.#.####.",
	"..........",
}

local function parse(rows)
	local grid, start, goal = {}, nil, nil
	for y, row in ipairs(rows) do
		grid[y] = {}
		for x = 1, #row do
			local c = row:sub(x, x)
			grid[y][x] = c ~= "#"
			if c == "S" then
				start = { x = x, y = y }
			elseif c == "G" then
				goal = { x = x, y = y }
			end
		end
	end
	return grid, start, goal
end

local function key(x, y)
	return y * 1000 + x
end

local function heuristic(a, b)
	return math.abs(a.x - b.x) + math.abs(a.y - b.y)
end

local Heap = {}
Heap.__index = Heap

function Heap.new()
	return setmetatable({ items = {}, size = 0 }, Heap)
end

function Heap:push(node, priority)
	self.size += 1
	local i = self.size
	self.items[i] = { node = node, priority = priority }
	while i > 1 do
		local parent = i // 2
		if self.items[parent].priority <= self.items[i].priority then
			break
		end
		self.items[parent], self.items[i] = self.items[i], self.items[parent]
		i = parent
	end
end

function Heap:pop()
	if self.size == 0 then
		return nil
	end
	local top = self.items[1]
	self.items[1] = self.items[self.size]
	self.items[self.size] = nil
	self.size -= 1
	local i = 1
	while true do
		local smallest, left, right = i, i * 2, i * 2 + 1
		if left <= self.size and self.items[left].priority < self.items[smallest].priority then
			smallest = left
		end
		if right <= self.size and self.items[right].priority < self.items[smallest].priority then
			smallest = right
		end
		if smallest == i then
			break
		end
		self.items[smallest], self.items[i] = self.items[i], self.items[smallest]
		i = smallest
	end
	return top.node
end

local DIRECTIONS = { { 1, 0 }, { -1, 0 }, { 0, 1 }, { 0, -1 } }

local function findPath(grid, start, goal)
	local open = Heap.new()
	local cameFrom, cost = {}, { [key(start.x, start.y)] = 0 }
	open:push(start, heuristic(start, goal))
	local visited = 0
	while open.size > 0 do
		local current = open:pop()
		visited += 1
		if current.x == goal.x and current.y == goal.y then
			local path = { current }
			local k = key(current.x, current.y)
			while cameFrom[k] do
				current = cameFrom[k]
				table.insert(path, 1, current)
				k = key(current.x, current.y)
			end
			return path, visited
		end
		for _, d in DIRECTIONS do
			local nx, ny = current.x + d[1], current.y + d[2]
			if not (grid[ny] and grid[ny][nx]) then
				continue
			end
			local nk = key(nx, ny)
			local newCost = cost[key(current.x, current.y)] + 1
			if cost[nk] == nil or newCost < cost[nk] then
				cost[nk] = newCost
				cameFrom[nk] = current
				local next = { x = nx, y = ny }
				open:push(next, newCost + heuristic(next, goal))
			end
		end
	end
	return nil, visited
end

local grid, start, goal = parse(MAP)
local path, visited = findPath(grid, start, goal)
if path then
	local steps = {}
	for i, p in path do
		steps[i] = `({p.x},{p.y})`
	end
	print(`found a path of {#path - 1} steps after visiting {visited} cells`)
	print(table.concat(steps, " "))
else
	print("no path")
end
