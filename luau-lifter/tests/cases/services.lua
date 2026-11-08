local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local RunService = game:GetService("RunService")
local Damage = ReplicatedStorage:WaitForChild("Remotes"):WaitForChild("Damage")
local Config = require(ReplicatedStorage:WaitForChild("Config"))
local LocalPlayer = Players.LocalPlayer
local v_u_1 = LocalPlayer.Character or LocalPlayer.CharacterAdded:Wait()
local Humanoid = v_u_1:FindFirstChildOfClass("Humanoid")
local part = Instance.new("Part")
part.Anchored = true
part.CanCollide = false
part.Size = Vector3.new(1, 1, 1)
part.Parent = workspace
local time = tick()
RunService.Heartbeat:Connect(function(_)
	-- upvalues: (copy) Humanoid, (copy) part, (copy) v_u_1, (copy) Config, (copy) time
	if Humanoid and Humanoid.Health > 0 then
		part.Position = v_u_1:GetPivot().Position + Vector3.new(0, Config.MarkerHeight, 0)
	end
	if tick() - time > Config.Lifetime then
		part:Destroy()
	end
end)
Damage.OnClientEvent:Connect(function(p2, p3)
	print(("took %d damage from %s"):format(p2, p3.Name))
end)