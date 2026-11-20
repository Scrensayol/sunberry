-- A camera shaker module: shakes stack, fade out over their duration and use Perlin noise.

local RunService = game:GetService("RunService")

local CameraShake = {}
CameraShake.__index = CameraShake

local NOISE_SPEED = 18

function CameraShake.new(camera)
	local self = setmetatable({}, CameraShake)
	self.camera = camera or workspace.CurrentCamera
	self.shakes = {}
	self.seed = math.random() * 1000
	self.connection = nil
	return self
end

function CameraShake:Shake(magnitude, duration, rotation)
	table.insert(self.shakes, {
		magnitude = magnitude,
		duration = duration,
		rotation = rotation or magnitude * 0.5,
		started = os.clock(),
	})
	if not self.connection then
		self.connection = RunService.RenderStepped:Connect(function()
			self:_step()
		end)
	end
end

function CameraShake:_offset(now)
	local position, rotation = Vector3.zero, Vector3.zero
	for i = #self.shakes, 1, -1 do
		local shake = self.shakes[i]
		local t = (now - shake.started) / shake.duration
		if t >= 1 then
			table.remove(self.shakes, i)
			continue
		end
		local fade = (1 - t) ^ 2
		local n = now * NOISE_SPEED
		position += Vector3.new(
			math.noise(n, self.seed, 0),
			math.noise(self.seed, n, 0),
			math.noise(0, self.seed, n)
		) * shake.magnitude * fade
		rotation += Vector3.new(math.noise(n, 0, self.seed), math.noise(0, n, self.seed), 0) * shake.rotation * fade
	end
	return position, rotation
end

function CameraShake:_step()
	local position, rotation = self:_offset(os.clock())
	if #self.shakes == 0 then
		self:Stop()
		return
	end
	self.camera.CFrame *= CFrame.new(position) * CFrame.Angles(math.rad(rotation.X), math.rad(rotation.Y), 0)
end

function CameraShake:Stop()
	table.clear(self.shakes)
	if self.connection then
		self.connection:Disconnect()
		self.connection = nil
	end
end

return CameraShake
