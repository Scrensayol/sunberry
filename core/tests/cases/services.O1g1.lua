local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local RunService = game:GetService("RunService")
local Remotes = ReplicatedStorage:WaitForChild("Remotes")
local Damage = Remotes:WaitForChild("Damage")
local Config = require(ReplicatedStorage:WaitForChild("Config"))
local LocalPlayer = Players.LocalPlayer
local Character = LocalPlayer.Character or LocalPlayer.CharacterAdded:Wait()
local Humanoid = Character:FindFirstChildOfClass("Humanoid")
local part
part = Instance.new("Part")
part.Anchored = true
part.CanCollide = false
part.Size = Vector3.new(1, 1, 1)
part.Parent = workspace
local v1
v1 = tick()
RunService.Heartbeat:Connect(function(_)
	if Humanoid and Humanoid.Health > 0 then
		part.Position = Character:GetPivot().Position + Vector3.new(0, Config.MarkerHeight, 0)
	end
	if tick() - v1 > Config.Lifetime then
		part:Destroy()
	end
end)
Damage.OnClientEvent:Connect(function(v2, v3)
	print(("took %d damage from %s"):format(v2, v3.Name))
end)
