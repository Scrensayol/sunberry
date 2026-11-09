local Players = game:GetService("Players")
local UserInputService = game:GetService("UserInputService")
local RunService = game:GetService("RunService")
local function onCharacter(character) -- name: onCharacter
	character:WaitForChild("Humanoid").Died:Connect(function()
		-- upvalues: (copy) character
		print(character.Name, "died")
	end)
end
Players.PlayerAdded:Connect(function(player)
	-- upvalues: (copy) onCharacter
	print("joined", player.Name, player.UserId)
	player.CharacterAdded:Connect(onCharacter)
	player.Chatted:Connect(function(message)
		-- upvalues: (copy) player
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
	-- upvalues: (copy) Players
	local v1 = Players:GetPlayerFromCharacter(hit.Parent)
	if v1 then
		print(v1.Name, "touched the pad")
	end
end)
UserInputService.InputBegan:Connect(function(input, gameProcessed)
	if not gameProcessed and input.KeyCode == Enum.KeyCode.E then
		print("interact")
	end
end)
local v_u_2 = 0
RunService.Heartbeat:Connect(function(deltaTime)
	-- upvalues: (ref) v_u_2
	v_u_2 += deltaTime
end)
local v3 = v_u_2
for _, child_2 in ipairs(workspace:GetChildren()) do
	print(child_2.Name)
end
for _, descendant in workspace:GetDescendants() do
	if not descendant:IsA("Light") then
		continue
	end
	descendant.Enabled = false
end
for _, player_2 in pairs(Players:GetPlayers()) do
	print(player_2.DisplayName, v3)
end