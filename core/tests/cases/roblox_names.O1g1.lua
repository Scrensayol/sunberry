local v1 = {}
v1.__index = v1
function v1.new(p2)
	local self = setmetatable({x = p2}, v1)
	local v3 = setmetatable({x = -p2}, v1)
	self.peer = v3
	function self.get()
		return self.x + v3.x
	end
	return self
end
function v1:clone()
	return (setmetatable({x = self.x}, v1))
end
function v1.spawner(p4)
	return function(p5)
		return (setmetatable({x = p4.x + p5}, v1))
	end
end
local v6 = setmetatable({}, {__index = v1})
v6.__index = v6
function v6.new(p7)
	local self = setmetatable(v1.new(p7), v6)
	self.sub = true
	return self
end
local function signal()
	local v8 = {}
	return {
		Connect = function(_, p9)
			table.insert(v8, p9)
		end,
		Fire = function(_, ...)
			for _, v in v8 do
				v(...)
			end
		end
	}
end
local v10 = {PlayerAdded = signal(), Stepped = signal()}
local v11 = {}
local function onStep(time, deltaTime)
	table.insert(v11, time + deltaTime)
end
v10.PlayerAdded:Connect(function(player)
	table.insert(v11, player .. " joined")
end)
v10.Stepped:Connect(onStep)
v10.PlayerAdded:Fire("luna")
v10.Stepped:Fire(1, 0.5)
local v12 = {
	GetChildren = function(_)
		return {"a", "b", "c"}
	end
}
for _, child in v12:GetChildren() do
	table.insert(v11, child)
end
for i, child2 in ipairs(v12:GetChildren()) do
	table.insert(v11, i .. child2)
end
local v13 = v1.new(2)
local v14 = v6.new(5)
print(v13.get(), v13:clone().x, v13:spawner()(3).x, v14.sub, v14.get(), table.concat(v11, ","))
