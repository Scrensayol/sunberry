local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local function onCharacter(v1)
	v1:WaitForChild("Humanoid").Died:Connect(function()
		print(v1.Name, "died")
	end)
end
Players.PlayerAdded:Connect(function(v2)
	print("joined", v2.Name, v2.UserId)
	v2.CharacterAdded:Connect(onCharacter)
	v2.Chatted:Connect(function(v3)
		if v3 == "/reset" and v2.Character then
			v2.Character:BreakJoints()
		end
	end)
end)
workspace.ChildAdded:Connect(function(v4)
	if v4:IsA("BasePart") then
		v4.Anchored = true
	end
end)
workspace:WaitForChild("Pad").Touched:Connect(function(v5)
	local v6 = Players:GetPlayerFromCharacter(v5.Parent)
	if v6 then
		print(v6.Name, "touched the pad")
	end
end)
UserInputService.InputBegan:Connect(function(v7, v8)
	if not v8 and v7.KeyCode == Enum.KeyCode.E then
		print("interact")
	end
end)
local v9 = 0
RunService.Heartbeat:Connect(function(v10)
	v9 += v10
end)
for _, v in ipairs(workspace:GetChildren()) do
	print(v.Name)
end
for _, v11 in workspace:GetDescendants() do
	if v11:IsA("Light") then
		v11.Enabled = false
	end
end
for _, v12 in pairs(Players:GetPlayers()) do
	print(v12.DisplayName, v9)
end
