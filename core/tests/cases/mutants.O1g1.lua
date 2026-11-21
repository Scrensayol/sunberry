local function curve(p1)
	return {
		IsA = function(_, p2)
			return p2 == "FloatCurve"
		end,
		GetKeys = function()
			return p1
		end
	}
end
local function hasKeys(p3)
	local X2 = p3.X
	local Y2 = p3.Y
	local Z2 = p3.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
X = curve({1})
print(hasKeys({X = 1, Y = 2, Z = 3}), hasKeys({}))
X = nil
local function find(p4, p5)
	for _ = 1, 1 do
		for _, v in p4 do
			if v > 10 and p5 then
				print("big", v)
				return found
			end
		end
	end
end
found = "yes"
print(find({1, 2, 30, 40}, true), find({1, 2}, false), find({1}, true))
