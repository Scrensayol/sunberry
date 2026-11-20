local v1 = {}
v1.__index = v1
local v2 = {
	sword = {weight = 8, stack = 1, value = 120},
	potion = {weight = 1, stack = 10, value = 25},
	arrow = {weight = 0.1, stack = 64, value = 1},
	shield = {weight = 12, stack = 1, value = 90}
}
function v1.new(p3)
	local self = setmetatable({}, v1)
	self.capacity = p3
	self.slots = {}
	self.weight = 0
	return self
end
local function round(p4, p5)
	local v6 = 10 ^ p5
	return math.floor(p4 * v6 + 0.5) / v6
end
function v1:add(p7, p8)
	local v9 = v2[p7]
	if not v9 then
		return false, `unknown item {p7}`
	end
	if self.weight + v9.weight * p8 > self.capacity then
		return false, `too heavy: {math.floor((self.weight + v9.weight * p8) * 10 + 0.5) / 10} > {self.capacity}`
	end
	local v10 = p8
	for _, v in self.slots do
		if v.name == p7 and v.count < v9.stack then
			local v11 = math.min(v9.stack - v.count, v10)
			v.count += v11
			v10 -= v11
			if v10 == 0 then
				break
			end
		end
	end
	while v10 > 0 do
		local v12 = math.min(v9.stack, v10)
		table.insert(self.slots, {name = p7, count = v12})
		v10 -= v12
	end
	self.weight += v9.weight * p8
	return true
end
function v1:remove(p13, p14)
	for i = #self.slots, 1, -1 do
		local v15 = self.slots[i]
		if v15.name == p13 then
			local v16 = math.min(v15.count, p14)
			v15.count -= v16
			p14 -= v16
			self.weight -= v2[p13].weight * v16
			if v15.count == 0 then
				table.remove(self.slots, i)
			end
			if p14 == 0 then
				return true
			end
		end
	end
	return p14 == 0
end
function v1:report()
	local v17 = {}
	for _, v3 in self.slots do
		v17[v3.name] = (v17[v3.name] or 0) + v3.count
	end
	local v18 = {}
	for v4 in v17 do
		table.insert(v18, v4)
	end
	table.sort(v18, function(p19, p20)
		return v2[p19].value * v17[p19] > v2[p20].value * v17[p20]
	end)
	local v21 = {}
	for k, v5 in v18 do
		v21[k] = `{k}. {v5} x{v17[v5]} ({v2[v5].value * v17[v5]}g)`
	end
	return table.concat(v21, "\n"), math.floor(self.weight * 100 + 0.5) / 100
end
local v22 = v1.new(40)
print(v22:add("arrow", 150))
print(v22:add("potion", 12))
print(v22:add("sword", 1))
print(v22:add("shield", 3))
print(v22:add("wand", 1))
print(v22:remove("arrow", 70), v22:remove("potion", 20))
print(v22:report())
