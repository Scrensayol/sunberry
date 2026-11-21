-- shapes Rosetta's mutation fuzzer found the new core refusing

-- locals a do block declares and drops: the if-expressions after it reuse their registers
-- for temporaries, so the locals' initializers still have to run first
local function curve(keys)
	return { IsA = function(_, class) return class == "FloatCurve" end, GetKeys = function() return keys end }
end
local function hasKeys(folder)
	do
		local X = folder.X
		local Y = folder.Y
		local Z = folder.Z
	end
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
X = curve({ 1 })
print(hasKeys({ X = 1, Y = 2, Z = 3 }), hasKeys({}))
X = nil

-- a helper -O2 inlines that returns from two loops deep: the inner loop sets the flag the
-- outer one declared and breaks, and the outer one breaks right after it
local function find(list, flag)
	for _ = 1, 1 do
		for _, v in list do
			if v > 10 then
				if flag then
					print("big", v)
					return found
				else
				end
			end
		end
	end
end
found = "yes"
print(find({ 1, 2, 30, 40 }, true), find({ 1, 2 }, false), find({ 1 }, true))
