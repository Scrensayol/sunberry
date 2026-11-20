local v1 = 0
local v2
function v2(p3)
	v1 += 1
	if p3 > 0 then
		v2(p3 - 1)
	end
end
local v4 = v2
v4(3)
local function isInside(parent, p5)
	while parent ~= nil do
		if parent.kind == p5 then
			return true
		end
		parent = parent.parent
	end
	return false
end
local v6 = {kind = "leaf", parent = {kind = "quote", parent = {kind = "doc"}}}
local function check(p7, p8)
	if p7 ~= nil and isInside(p7, p8) then
		return "inside " .. p8
	end
	return "outside " .. p8
end
local function isSupported(p9, p10)
	if type(p9) == "table" then
		local kind = p9.kind
		if kind == 1 or kind == 2 or kind == 3 then
			return true
		end
	end
	if p10 == 4 or p10 == 5 then
		return true
	end
	return false
end
local function pick(p11)
	if p11 == nil then
		return nil
	end
	if not isSupported(p11.payload, p11.request and p11.request.kind) then
		return nil
	end
	return p11.price
end
local function scan(p12, p13)
	if #p12 > 0 then
		local v14 = 0
		repeat
			v14 += 1
			if p12[v14] == p13 then
				v1 += 100
				return
			end
		until #p12 <= v14
		return
	end
	if p13 then
		v1 += 1
	end
end
scan({1, 2, 3}, 2)
scan({}, true)
local function curve(p15)
	local v16 = {}
	local function IsA(_, p17)
		return p17 == "FloatCurve"
	end
	v16.IsA = IsA
	local function GetKeys()
		return p15
	end
	v16.GetKeys = GetKeys
	return v16
end
local function hasKeys(p18)
	local X = p18.X
	local Y = p18.Y
	local Z = p18.Z
	local v19 = if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false
	if not v19 then
		v19 = if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false
		if not v19 then
			if Z then
				v19 = Z:IsA("FloatCurve")
				if v19 then
					v19 = #Z:GetKeys() > 0
				end
				return v19
			end
			v19 = false
		end
	end
	return v19
end
print(hasKeys({X = curve({}), Z = curve({1})}), hasKeys({}), hasKeys({Y = curve({1, 2})}))
local v20 = v1 > 3
print(v1, check(v6, "quote"), check(v6, "table"), check(nil, "doc"))
print(pick({price = 7, payload = {kind = 2}}), pick({payload = 1, price = 8, request = {kind = 5}}), pick({price = 9, payload = {}}))
local v21 = print
if v20 then
end
local v22 = {Asset = 1, Bundle = 2, Look = 3}
local v23 = {GamePass = 4, Product = 5}
local v24 = v20
if v1 > 1000 then
	if v20 then
	end
end
v21(v24, "no")
