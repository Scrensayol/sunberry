local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local function v2(v1)
	v1:WaitForChild("Humanoid").Died:Connect(function()
		print(v1.Name, "died")
	end)
end
Players.PlayerAdded:Connect(function(v3)
	print("joined", v3.Name, v3.UserId)
	v3.CharacterAdded:Connect(v2)
	v3.Chatted:Connect(function(v4)
		if v4 == "/reset" and v3.Character then
			v3.Character:BreakJoints()
		end
	end)
end)
workspace.ChildAdded:Connect(function(v5)
	if v5:IsA("BasePart") then
		v5.Anchored = true
	end
end)
workspace:WaitForChild("Pad").Touched:Connect(function(v6)
	local v7 = Players:GetPlayerFromCharacter(v6.Parent)
	if v7 then
		print(v7.Name, "touched the pad")
	end
end)
UserInputService.InputBegan:Connect(function(v8, v9)
	if not v9 and v8.KeyCode == Enum.KeyCode.E then
		print("interact")
	end
end)
local v10 = 0
RunService.Heartbeat:Connect(function(v11)
	v10 = v10 + v11
end)
for i, v in ipairs(workspace:GetChildren()) do
	print(v.Name)
end
for k, v12 in workspace:GetDescendants() do
	if v12:IsA("Light") then
		v12.Enabled = false
	end
end
for k2, v13 in pairs(Players:GetPlayers()) do
	print(v13.DisplayName, v10)
end
