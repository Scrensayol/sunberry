local function curve(keys)
	return {
		IsA = function(_, class)
			return class == "FloatCurve"
		end,
		GetKeys = function()
			return keys
		end
	}
end
local function hasKeys(folder)
	local X2 = folder.X
	local Y2 = folder.Y
	local Z2 = folder.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
X = curve({1})
print(hasKeys({X = 1, Y = 2, Z = 3}), (hasKeys({})))
X = nil
local function find(list, flag)
	for _ = 1, 1 do
		for _, v in list do
			if v > 10 and flag then
				print("big", v)
				return found
			end
		end
	end
end
found = "yes"
local v1 = print
local list2 = {1, 2, 30, 40}
local v2 = false
local v3
for _ = 1, 1 do
	for _, v4 in list2 do
		if v4 > 10 then
			print("big", v4)
			v3 = found
			v2 = true
			break
		end
	end
	if v2 then
		break
	end
end
if not v2 then
	v3 = nil
end
for _, v5 in {1, 2} do
	if v5 > 10 then
	end
end
v1(v3, nil, find({1}, true))
