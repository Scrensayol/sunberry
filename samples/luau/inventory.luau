--!strict
-- An inventory with stacking, weight limits and a sorted report.

local Inventory = {}
Inventory.__index = Inventory

local ITEMS = {
	sword = { weight = 8, stack = 1, value = 120 },
	potion = { weight = 1, stack = 10, value = 25 },
	arrow = { weight = 0.1, stack = 64, value = 1 },
	shield = { weight = 12, stack = 1, value = 90 },
}

function Inventory.new(capacity)
	local self = setmetatable({}, Inventory)
	self.capacity = capacity
	self.slots = {}
	self.weight = 0
	return self
end

local function round(n, places)
	local scale = 10 ^ places
	return math.floor(n * scale + 0.5) / scale
end

function Inventory:add(name, count)
	local info = ITEMS[name]
	if not info then
		return false, `unknown item {name}`
	end
	if self.weight + info.weight * count > self.capacity then
		return false, `too heavy: {round(self.weight + info.weight * count, 1)} > {self.capacity}`
	end
	local left = count
	for _, slot in self.slots do
		if slot.name == name and slot.count < info.stack then
			local moved = math.min(info.stack - slot.count, left)
			slot.count += moved
			left -= moved
			if left == 0 then
				break
			end
		end
	end
	while left > 0 do
		local moved = math.min(info.stack, left)
		table.insert(self.slots, { name = name, count = moved })
		left -= moved
	end
	self.weight += info.weight * count
	return true
end

function Inventory:remove(name, count)
	for i = #self.slots, 1, -1 do
		local slot = self.slots[i]
		if slot.name ~= name then
			continue
		end
		local taken = math.min(slot.count, count)
		slot.count -= taken
		count -= taken
		self.weight -= ITEMS[name].weight * taken
		if slot.count == 0 then
			table.remove(self.slots, i)
		end
		if count == 0 then
			return true
		end
	end
	return count == 0
end

function Inventory:report()
	local totals = {}
	for _, slot in self.slots do
		totals[slot.name] = (totals[slot.name] or 0) + slot.count
	end
	local names = {}
	for name in totals do
		table.insert(names, name)
	end
	table.sort(names, function(a, b)
		return ITEMS[a].value * totals[a] > ITEMS[b].value * totals[b]
	end)
	local lines = {}
	for rank, name in names do
		lines[rank] = `{rank}. {name} x{totals[name]} ({ITEMS[name].value * totals[name]}g)`
	end
	return table.concat(lines, "\n"), round(self.weight, 2)
end

local bag = Inventory.new(40)
print(bag:add("arrow", 150))
print(bag:add("potion", 12))
print(bag:add("sword", 1))
print(bag:add("shield", 3))
print(bag:add("wand", 1))
print(bag:remove("arrow", 70), bag:remove("potion", 20))
print(bag:report())
