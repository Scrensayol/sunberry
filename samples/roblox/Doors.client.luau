-- Opens every part tagged "Door" with a proximity prompt and a tween, and tells the server.

local CollectionService = game:GetService("CollectionService")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local TweenService = game:GetService("TweenService")

local doorUsed = ReplicatedStorage:WaitForChild("Remotes"):WaitForChild("DoorUsed")
local TWEEN = TweenInfo.new(0.6, Enum.EasingStyle.Quad, Enum.EasingDirection.Out)
local OPEN_TIME = 3

local busy = {}

local function setOpen(door, open)
	local closed = door:GetAttribute("ClosedCFrame") or door.CFrame
	door:SetAttribute("ClosedCFrame", closed)
	local goal = if open then closed * CFrame.new(0, 0, -door.Size.Z) * CFrame.Angles(0, math.rad(90), 0) else closed
	local tween = TweenService:Create(door, TWEEN, { CFrame = goal })
	tween:Play()
	return tween
end

local function setup(door)
	if not door:IsA("BasePart") then
		return
	end
	local prompt = Instance.new("ProximityPrompt")
	prompt.ActionText = "Open"
	prompt.ObjectText = door.Name
	prompt.HoldDuration = door:GetAttribute("Locked") and 1.5 or 0
	prompt.Parent = door

	prompt.Triggered:Connect(function()
		if busy[door] then
			return
		end
		busy[door] = true
		prompt.Enabled = false
		doorUsed:FireServer(door)
		setOpen(door, true).Completed:Wait()
		task.wait(OPEN_TIME)
		setOpen(door, false).Completed:Wait()
		prompt.Enabled = true
		busy[door] = nil
	end)
end

for _, door in CollectionService:GetTagged("Door") do
	setup(door)
end
CollectionService:GetInstanceAddedSignal("Door"):Connect(setup)
CollectionService:GetInstanceRemovedSignal("Door"):Connect(function(door)
	busy[door] = nil
	local prompt = door:FindFirstChildOfClass("ProximityPrompt")
	if prompt then
		prompt:Destroy()
	end
end)
