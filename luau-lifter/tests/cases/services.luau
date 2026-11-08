-- naming hints: services, children, modules, instances
local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local RunService = game:GetService("RunService")

local remotes = ReplicatedStorage:WaitForChild("Remotes")
local damageEvent = remotes:WaitForChild("Damage")
local config = require(ReplicatedStorage:WaitForChild("Config"))

local player = Players.LocalPlayer
local character = player.Character or player.CharacterAdded:Wait()
local humanoid = character:FindFirstChildOfClass("Humanoid")

local marker = Instance.new("Part")
marker.Anchored = true
marker.CanCollide = false
marker.Size = Vector3.new(1, 1, 1)
marker.Parent = workspace

local started = tick()
RunService.Heartbeat:Connect(function(dt)
	if humanoid and humanoid.Health > 0 then
		marker.Position = character:GetPivot().Position + Vector3.new(0, config.MarkerHeight, 0)
	end
	if tick() - started > config.Lifetime then
		marker:Destroy()
	end
end)

damageEvent.OnClientEvent:Connect(function(amount, source)
	print(("took %d damage from %s"):format(amount, source.Name))
end)
