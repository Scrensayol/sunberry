local CollectionService = game:GetService("CollectionService")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local TweenService = game:GetService("TweenService")
local DoorUsed = ReplicatedStorage:WaitForChild("Remotes"):WaitForChild("DoorUsed")
local v1 = TweenInfo.new(0.6, Enum.EasingStyle.Quad, Enum.EasingDirection.Out)
local v2 = {}
local function setOpen(p3, p4)
	local v5 = p3:GetAttribute("ClosedCFrame") or p3.CFrame
	p3:SetAttribute("ClosedCFrame", v5)
	local v6 = if p4 then v5 * CFrame.new(0, 0, -p3.Size.Z) * CFrame.Angles(0, 1.5707963267948966, 0) else v5
	local v7 = TweenService:Create(p3, v1, {CFrame = v6})
	v7:Play()
	return v7
end
local function setup(p8)
	if not p8:IsA("BasePart") then
		return
	end
	local proximityPrompt
	proximityPrompt = Instance.new("ProximityPrompt")
	proximityPrompt.ActionText = "Open"
	proximityPrompt.ObjectText = p8.Name
	proximityPrompt.HoldDuration = if p8:GetAttribute("Locked") then 1.5 else 0
	proximityPrompt.Parent = p8
	proximityPrompt.Triggered:Connect(function()
		if v2[p8] then
			return
		end
		v2[p8] = true
		proximityPrompt.Enabled = false
		DoorUsed:FireServer(p8)
		setOpen(p8, true).Completed:Wait()
		task.wait(3)
		setOpen(p8, false).Completed:Wait()
		proximityPrompt.Enabled = true
		v2[p8] = nil
	end)
end
for _, instance in CollectionService:GetTagged("Door") do
	setup(instance)
end
CollectionService:GetInstanceAddedSignal("Door"):Connect(setup)
CollectionService:GetInstanceRemovedSignal("Door"):Connect(function(p9)
	v2[p9] = nil
	local ProximityPrompt = p9:FindFirstChildOfClass("ProximityPrompt")
	if ProximityPrompt then
		ProximityPrompt:Destroy()
	end
end)
