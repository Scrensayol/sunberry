-- Saves each player's coins and wins, with retries, autosave and a save on shutdown.

local Players = game:GetService("Players")
local DataStoreService = game:GetService("DataStoreService")
local RunService = game:GetService("RunService")

local store = DataStoreService:GetDataStore("PlayerStats_v2")
local DEFAULTS = { Coins = 100, Wins = 0 }
local AUTOSAVE_INTERVAL = 120
local MAX_ATTEMPTS = 4

local sessions = {}

local function withRetries(label, callback)
	local lastError
	for attempt = 1, MAX_ATTEMPTS do
		local ok, result = pcall(callback)
		if ok then
			return true, result
		end
		lastError = result
		warn(`{label} failed (attempt {attempt}/{MAX_ATTEMPTS}): {result}`)
		task.wait(2 ^ attempt)
	end
	return false, lastError
end

local function setupLeaderstats(player, data)
	local leaderstats = Instance.new("Folder")
	leaderstats.Name = "leaderstats"
	for statName, value in data do
		local stat = Instance.new("IntValue")
		stat.Name = statName
		stat.Value = value
		stat.Parent = leaderstats
	end
	leaderstats.Parent = player
end

local function save(player)
	local data = sessions[player]
	if not data then
		return
	end
	local leaderstats = player:FindFirstChild("leaderstats")
	if leaderstats then
		for statName in DEFAULTS do
			local stat = leaderstats:FindFirstChild(statName)
			if stat then
				data[statName] = stat.Value
			end
		end
	end
	withRetries(`save {player.UserId}`, function()
		store:UpdateAsync(`player_{player.UserId}`, function()
			return data
		end)
	end)
end

Players.PlayerAdded:Connect(function(player)
	local ok, saved = withRetries(`load {player.UserId}`, function()
		return store:GetAsync(`player_{player.UserId}`)
	end)
	if not ok then
		player:Kick("Couldn't load your data, please rejoin.")
		return
	end
	local data = table.clone(DEFAULTS)
	if type(saved) == "table" then
		for key, value in saved do
			if DEFAULTS[key] ~= nil then
				data[key] = value
			end
		end
	end
	sessions[player] = data
	setupLeaderstats(player, data)
end)

Players.PlayerRemoving:Connect(function(player)
	save(player)
	sessions[player] = nil
end)

task.spawn(function()
	while true do
		task.wait(AUTOSAVE_INTERVAL)
		for _, player in Players:GetPlayers() do
			task.spawn(save, player)
		end
	end
end)

game:BindToClose(function()
	if RunService:IsStudio() then
		return
	end
	for _, player in Players:GetPlayers() do
		save(player)
	end
end)
