local Players = game:GetService("Players")
local DataStoreService = game:GetService("DataStoreService")
local RunService = game:GetService("RunService")
local v1 = DataStoreService:GetDataStore("PlayerStats_v2")
local v2 = {Coins = 100, Wins = 0}
local v3 = {}
local function withRetries(p4, p5)
	local v6
	for i = 1, 4 do
		local success, result = pcall(p5)
		if success then
			return true, result
		end
		v6 = result
		warn(`{p4} failed (attempt {i}/{4}): {result}`)
		task.wait(2 ^ i)
	end
	return false, v6
end
local function setupLeaderstats(p7, p8)
	local folder = Instance.new("Folder")
	folder.Name = "leaderstats"
	for k, v in p8 do
		local intValue = Instance.new("IntValue")
		intValue.Name = k
		intValue.Value = v
		intValue.Parent = folder
	end
	folder.Parent = p7
end
local function save(p9)
	local v10 = v3[p9]
	if not v10 then
		return
	end
	local leaderstats = p9:FindFirstChild("leaderstats")
	if leaderstats then
		for v4 in v2 do
			local v11 = leaderstats:FindFirstChild(v4)
			if v11 then
				v10[v4] = v11.Value
			end
		end
	end
	withRetries(`save {p9.UserId}`, function()
		v1:UpdateAsync(`player_{p9.UserId}`, function()
			return v10
		end)
	end)
end
Players.PlayerAdded:Connect(function(player)
	local v12, v13 = withRetries(`load {player.UserId}`, function()
		return v1:GetAsync(`player_{player.UserId}`)
	end)
	if not v12 then
		player:Kick("Couldn't load your data, please rejoin.")
		return
	end
	local v14 = table.clone(v2)
	if type(v13) == "table" then
		for k2, v5 in v13 do
			if v2[k2] ~= nil then
				v14[k2] = v5
			end
		end
	end
	v3[player] = v14
	setupLeaderstats(player, v14)
end)
Players.PlayerRemoving:Connect(function(player2)
	save(player2)
	v3[player2] = nil
end)
task.spawn(function()
	while true do
		task.wait(120)
		for _, player3 in Players:GetPlayers() do
			task.spawn(save, player3)
		end
	end
end)
game:BindToClose(function()
	if RunService:IsStudio() then
		return
	end
	for _, player4 in Players:GetPlayers() do
		save(player4)
	end
end)
