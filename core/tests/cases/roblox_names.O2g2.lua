local Class = {}
Class.__index = Class
function Class.new(x)
	local obj = setmetatable({x = x}, Class)
	local other = setmetatable({x = -x}, Class)
	obj.peer = other
	function obj.get()
		return obj.x + other.x
	end
	return obj
end
function Class:clone()
	local copy = setmetatable({x = self.x}, Class)
	return copy
end
function Class:spawner()
	return function(n)
		local inner = setmetatable({x = self.x + n}, Class)
		return inner
	end
end
local Sub = setmetatable({}, {__index = Class})
Sub.__index = Sub
function Sub.new(x2)
	local obj2 = setmetatable(Class.new(x2), Sub)
	obj2.sub = true
	return obj2
end
local function signal()
	local handlers = {}
	return {
		Connect = function(_, f)
			table.insert(handlers, f)
		end,
		Fire = function(_, ...)
			for _, f2 in handlers do
				f2(...)
			end
		end
	}
end
local Players = {PlayerAdded = signal(), Stepped = signal()}
local log = {}
local function onStep(t, dt)
	table.insert(log, t + dt)
end
Players.PlayerAdded:Connect(function(plr)
	table.insert(log, plr .. " joined")
end)
Players.Stepped:Connect(onStep)
Players.PlayerAdded:Fire("luna")
Players.Stepped:Fire(1, 0.5)
local folder = {}
function folder:GetChildren()
	return {"a", "b", "c"}
end
for _, item in folder:GetChildren() do
	table.insert(log, item)
end
for index, item2 in ipairs(folder:GetChildren()) do
	table.insert(log, index .. item2)
end
local a = Class.new(2)
local b = Sub.new(5)
print(a.get(), a:clone().x, a:spawner()(3).x, b.sub, b.get(), table.concat(log, ","))
