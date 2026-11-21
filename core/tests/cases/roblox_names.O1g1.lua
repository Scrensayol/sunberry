local v1 = {}
v1.__index = v1
function v1.new(x)
	local self = setmetatable({x = x}, v1)
	local v2 = setmetatable({x = -x}, v1)
	self.peer = v2
	function self.get()
		return self.x + v2.x
	end
	return self
end
function v1:clone()
	return (setmetatable({x = self.x}, v1))
end
function v1:spawner()
	return function(p3)
		return (setmetatable({x = self.x + p3}, v1))
	end
end
local v4 = setmetatable({}, {__index = v1})
v4.__index = v4
function v4.new(p5)
	local self = setmetatable(v1.new(p5), v4)
	self.sub = true
	return self
end
local function signal()
	local v6 = {}
	return {
		Connect = function(_, p7)
			table.insert(v6, p7)
		end,
		Fire = function(_, ...)
			for _, v in v6 do
				v(...)
			end
		end
	}
end
local v8 = {PlayerAdded = signal(), Stepped = signal()}
local v9 = {}
local function onStep(time, deltaTime)
	table.insert(v9, time + deltaTime)
end
v8.PlayerAdded:Connect(function(player)
	table.insert(v9, player .. " joined")
end)
v8.Stepped:Connect(onStep)
v8.PlayerAdded:Fire("luna")
v8.Stepped:Fire(1, 0.5)
local v10 = {
	GetChildren = function(_)
		return {"a", "b", "c"}
	end
}
for _, child in v10:GetChildren() do
	table.insert(v9, child)
end
for i, child2 in ipairs(v10:GetChildren()) do
	table.insert(v9, i .. child2)
end
local v11 = v1.new(2)
local v12 = v4.new(5)
print(v11.get(), v11:clone().x, v11:spawner()(3).x, v12.sub, v12.get(), table.concat(v9, ","))
