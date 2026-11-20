local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local function onCharacter(character)
	local humanoid = character:WaitForChild("Humanoid")
	humanoid.Died:Connect(function()
		print(character.Name, "died")
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
local pad = workspace:WaitForChild("Pad")
pad.Touched:Connect(function(hit)
	local owner = Players:GetPlayerFromCharacter(hit.Parent)
	if owner then
		print(owner.Name, "touched the pad")
	end
end)
UserInputService.InputBegan:Connect(function(input, gameProcessed)
	if not gameProcessed and input.KeyCode == Enum.KeyCode.E then
		print("interact")
	end
end)
local elapsed = 0
RunService.Heartbeat:Connect(function(deltaTime)
	elapsed += deltaTime
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
	print(player2.DisplayName, elapsed)
end
