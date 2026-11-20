local RunService = game:GetService("RunService")
local CameraShake = {}
CameraShake.__index = CameraShake
function CameraShake.new(p1)
	local self = setmetatable({}, CameraShake)
	self.camera = p1 or workspace.CurrentCamera
	self.shakes = {}
	self.seed = math.random() * 1000
	self.connection = nil
	return self
end
function CameraShake:Shake(p2, p3, p4)
	table.insert(self.shakes, {magnitude = p2, duration = p3, rotation = p4 or p2 * 0.5, started = os.clock()})
	if not self.connection then
		self.connection = RunService.RenderStepped:Connect(function()
			self:_step()
		end)
	end
end
function CameraShake:_offset(p5)
	local zero = Vector3.zero
	local zero2 = Vector3.zero
	for i = #self.shakes, 1, -1 do
		local v6 = self.shakes[i]
		local v7 = (p5 - v6.started) / v6.duration
		if v7 >= 1 then
			table.remove(self.shakes, i)
		else
			local v8 = (1 - v7) ^ 2
			local v9 = p5 * 18
			zero += Vector3.new(math.noise(v9, self.seed, 0), math.noise(self.seed, v9, 0), math.noise(0, self.seed, v9)) * v6.magnitude * v8
			zero2 += Vector3.new(math.noise(v9, 0, self.seed), math.noise(0, v9, self.seed), 0) * v6.rotation * v8
		end
	end
	return zero, zero2
end
function CameraShake:_step()
	local v10, v11 = self:_offset(os.clock())
	if #self.shakes == 0 then
		self:Stop()
		return
	end
	local camera = self.camera
	camera.CFrame *= CFrame.new(v10) * CFrame.Angles(math.rad(v11.X), math.rad(v11.Y), 0)
end
function CameraShake:Stop()
	table.clear(self.shakes)
	if self.connection then
		self.connection:Disconnect()
		self.connection = nil
	end
end
return CameraShake
