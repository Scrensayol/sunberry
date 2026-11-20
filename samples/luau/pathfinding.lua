local function parse(p1)
	local v2 = {}
	local v3
	local v4
	for i, v in ipairs(p1) do
		v2[i] = {}
		for i2 = 1, #v do
			local v5 = v:sub(i2, i2)
			v2[i][i2] = v5 ~= "#"
			if v5 == "S" then
				v3 = {x = i2, y = i}
			elseif v5 == "G" then
				v4 = {x = i2, y = i}
			end
		end
	end
	return v2, v3, v4
end
local function key(p6, p7)
	return p7 * 1000 + p6
end
local function heuristic(p8, p9)
	return math.abs(p8.x - p9.x) + math.abs(p8.y - p9.y)
end
local v10 = {}
v10.__index = v10
function v10.new()
	return (setmetatable({size = 0, items = {}}, v10))
end
function v10:push(p11, p12)
	self.size += 1
	local size = self.size
	self.items[size] = {node = p11, priority = p12}
	while size > 1 do
		local v13 = size // 2
		if self.items[v13].priority <= self.items[size].priority then
			break
		end
		local items = self.items
		local items2 = self.items
		local v14 = self.items[size]
		local v15 = self.items[v13]
		items[v13] = v14
		items2[size] = v15
		size = v13
	end
end
function v10:pop()
	if self.size == 0 then
		return nil
	end
	local v16 = self.items[1]
	self.items[1] = self.items[self.size]
	self.items[self.size] = nil
	self.size -= 1
	local v17 = 1
	while true do
		local v18 = v17
		local v19 = v17 * 2
		local v20 = v17 * 2 + 1
		if v19 <= self.size and self.items[v19].priority < self.items[v18].priority then
			v18 = v19
		end
		if v20 <= self.size and self.items[v20].priority < self.items[v18].priority then
			v18 = v20
		end
		if v18 == v17 then
			break
		end
		local items3 = self.items
		local items4 = self.items
		local v21 = self.items[v17]
		local v22 = self.items[v18]
		items3[v18] = v21
		items4[v17] = v22
		v17 = v18
	end
	return v16.node
end
local v23 = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}}
local v24 = {
	"S....#....",
	".##..#.##.",
	".#...#..#.",
	".#.###..#.",
	"...#....#G",
	"##.#.####.",
	".........."
}
local function findPath(p25, p26, p27)
	local v28 = v10.new()
	local v29 = {[key(p26.x, p26.y)] = 0}
	v28:push(p26, (heuristic(p26, p27)))
	local v30 = {}
	local v31 = 0
	while v28.size > 0 do
		local v32 = v28:pop()
		v31 += 1
		if v32.x == p27.x and v32.y == p27.y then
			local v33 = {v32}
			local v34 = key(v32.x, v32.y)
			while v30[v34] do
				v32 = v30[v34]
				table.insert(v33, 1, v32)
				v34 = key(v32.x, v32.y)
			end
			return v33, v31
		end
		for _, v6 in v23 do
			local v35 = v32.x + v6[1]
			local v36 = v32.y + v6[2]
			if p25[v36] and p25[v36][v35] then
				local v37 = key(v35, v36)
				local v38 = v29[key(v32.x, v32.y)] + 1
				if v29[v37] == nil or v38 < v29[v37] then
					v29[v37] = v38
					v30[v37] = v32
					local v39 = {x = v35, y = v36}
					v28:push(v39, v38 + heuristic(v39, p27))
				end
			end
		end
	end
	return nil, v31
end
local v40, v41, v42 = parse(v24)
local v43, v44 = findPath(v40, v41, v42)
if v43 then
	local v45 = {}
	for k, v7 in v43 do
		v45[k] = `({v7.x},{v7.y})`
	end
	print(`found a path of {#v43 - 1} steps after visiting {v44} cells`)
	print(table.concat(v45, " "))
	return
end
print("no path")
