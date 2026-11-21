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
local function isSupported(payload, p9)
	if type(payload) == "table" then
		local kind = payload.kind
		if kind == 1 or kind == 2 or kind == 3 then
			return true
		end
	end
	if p9 == 4 or p9 == 5 then
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
local function scan(p10, p11)
	if #p10 > 0 then
		local v12 = 0
		repeat
			v12 += 1
			if p10[v12] == p11 then
				v1 += 100
				return
			end
		until #p10 <= v12
		return
	end
	if p11 then
		v1 += 1
	end
end
scan({1, 2, 3}, 2)
scan({}, true)
local function curve(p13)
	return {
		IsA = function(_, p14)
			return p14 == "FloatCurve"
		end,
		GetKeys = function()
			return p13
		end
	}
end
local function hasKeys(p15)
	local X = p15.X
	local Y = p15.Y
	local Z = p15.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
print(hasKeys({X = curve({}), Z = curve({1})}), hasKeys({}), hasKeys({Y = curve({1, 2})}))
local v16 = v1 > 3
print(v1, check(v6, "quote"), check(v6, "table"), check(nil, "doc"))
print(
	pick({price = 7, payload = {kind = 2}}),
	pick({payload = 1, price = 8, request = {kind = 5}}),
	pick({price = 9, payload = {}})
)
local v17 = print
local v18 = {Asset = 1, Bundle = 2, Look = 3}
local v19 = {GamePass = 4, Product = 5}
local v20 = v16
if v1 > 1000 then
end
v17(v20, "no")
