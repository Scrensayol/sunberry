local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local function onCharacter(p1)
	p1:WaitForChild("Humanoid").Died:Connect(function()
		print(p1.Name, "died")
	end)
end
Players.PlayerAdded:Connect(function(player)
	print("joined", player.Name, player.UserId)
	player.CharacterAdded:Connect(onCharacter)
	player.Chatted:Connect(function(message)
		if message == "/reset" and player.Character then
			player.Character:BreakJoints()
		end
	end)
end)
workspace.ChildAdded:Connect(function(child)
	if child:IsA("BasePart") then
		child.Anchored = true
	end
end)
workspace:WaitForChild("Pad").Touched:Connect(function(hit)
	local v2 = Players:GetPlayerFromCharacter(hit.Parent)
	if v2 then
		print(v2.Name, "touched the pad")
	end
end)
UserInputService.InputBegan:Connect(function(input, gameProcessed)
	if not gameProcessed and input.KeyCode == Enum.KeyCode.E then
		print("interact")
	end
end)
local v3 = 0
RunService.Heartbeat:Connect(function(deltaTime)
	v3 += deltaTime
end)
for _, child2 in ipairs(workspace:GetChildren()) do
	print(child2.Name)
end
for _, descendant in workspace:GetDescendants() do
	if descendant:IsA("Light") then
		descendant.Enabled = false
	end
end
for _, player2 in pairs(Players:GetPlayers()) do
	print(player2.DisplayName, v3)
end
