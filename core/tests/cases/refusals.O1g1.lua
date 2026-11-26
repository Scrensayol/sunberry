local v1 = 0
local v2
function v2(p3)
	v1 += 1
	if p3 > 0 then
		v2(p3 - 1)
	end
end
v2(3)
local function isInside(parent, p4)
	while parent ~= nil do
		if parent.kind == p4 then
			return true
		end
		parent = parent.parent
	end
	return false
end
local v5 = {kind = "leaf", parent = {kind = "quote", parent = {kind = "doc"}}}
local function check(p6, p7)
	if p6 ~= nil and isInside(p6, p7) then
		return "inside " .. p7
	end
	return "outside " .. p7
end
local function isSupported(payload, p8)
	if type(payload) == "table" then
		local kind = payload.kind
		if kind == 1 or kind == 2 or kind == 3 then
			return true
		end
	end
	if p8 == 4 or p8 == 5 then
		return true
	end
	return false
end
local function pick(action)
	if action == nil then
		return nil
	end
	if not isSupported(action.payload, action.request and action.request.kind) then
		return nil
	end
	return action.price
end
local function scan(p9, p10)
	if #p9 > 0 then
		local v11 = 0
		repeat
			v11 += 1
			if p9[v11] == p10 then
				v1 += 100
				return
			end
		until #p9 <= v11
		return
	end
	if p10 then
		v1 += 1
	end
end
scan({1, 2, 3}, 2)
scan({}, true)
local function curve(p12)
	return {
		IsA = function(_, p13)
			return p13 == "FloatCurve"
		end,
		GetKeys = function()
			return p12
		end
	}
end
local function hasKeys(p14)
	local X = p14.X
	local Y = p14.Y
	local Z = p14.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
print(hasKeys({X = curve({}), Z = curve({1})}), hasKeys({}), hasKeys({Y = curve({1, 2})}))
local v15 = v1 > 3
print(v1, check(v5, "quote"), check(v5, "table"), check(nil, "doc"))
print(
	pick({payload = {kind = 2}, price = 7}),
	pick({payload = 1, request = {kind = 5}, price = 8}),
	pick({payload = {}, price = 9})
)
local v16 = print
local v17 = {Asset = 1, Bundle = 2, Look = 3}
local v18 = {GamePass = 4, Product = 5}
local v19 = v15
if v1 > 1000 then
end
v16(v19, "no")
