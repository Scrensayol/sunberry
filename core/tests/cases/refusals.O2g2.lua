local count = 0
local tick
function tick(n)
	count += 1
	if n > 0 then
		tick(n - 1)
	end
end
tick(3)
local function isInside(node, kind)
	while node ~= nil do
		if node.kind == kind then
			return true
		end
		node = node.parent
	end
	return false
end
local tree = {kind = "leaf", parent = {kind = "quote", parent = {kind = "doc"}}}
local function check(start, kind2)
	local found = start ~= nil
	if found and isInside(start, kind2) then
		return "inside " .. kind2
	end
	return "outside " .. kind2
end
local A = {Asset = 1, Bundle = 2, Look = 3}
local B = {GamePass = 4, Product = 5}
local function isSupported(payload, requestType)
	if type(payload) == "table" then
		local kind3 = payload.kind
		if kind3 == 1 or kind3 == 2 or kind3 == 3 then
			return true
		end
	end
	if requestType == 4 or requestType == 5 then
		return true
	end
	return false
end
local function pick(state)
	if state == nil then
		return nil
	end
	local payload2 = state.payload
	local requestType2 = state.request and state.request.kind
	local kind4
	local v1
	local v2 = type(payload2) == "table"
	if v2 then
		kind4 = payload2.kind
		v2 = kind4 == A.Asset or kind4 == A.Bundle or kind4 == A.Look
	end
	local v3 = not v2
	if not v3 then
		v1 = true
		v3 = false
	end
	local v4 = v3
	if v4 then
		local v5 = not (requestType2 == B.GamePass or requestType2 == B.Product)
		if not v5 then
			v1 = true
			v5 = false
		end
		v4 = v5
	end
	if v4 then
		v1 = false
	end
	if not v1 then
		return nil
	end
	return state.price
end
local function scan(list, limit)
	if #list > 0 then
		local i = 0
		repeat
			i += 1
			if list[i] == limit then
				count += 100
				return
			end
		until #list <= i
		return
	end
	if limit then
		count += 1
	end
end
local list2 = {1, 2, 3}
if #list2 > 0 then
	local i2 = 0
	repeat
		i2 += 1
		if list2[i2] == 2 then
			count += 100
			break
		end
	until #list2 <= i2
else
	count += 1
end
local list3 = {}
if #list3 > 0 then
	local i3 = 0
	repeat
		i3 += 1
		if list3[i3] == true then
			count += 100
			break
		end
	until #list3 <= i3
else
	count += 1
end
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
	local X = folder.X
	local Y = folder.Y
	local Z = folder.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
print(hasKeys({X = curve({}), Z = curve({1})}), hasKeys({}), (hasKeys({Y = curve({1, 2})})))
local flag = count > 3
local v6 = print
local v7 = count
local v8 = check(tree, "quote")
local v9 = check(tree, "table")
local v10 = false
v6(v7, v8, v9, "outside doc")
print(
	pick({payload = {kind = 2}, price = 7}),
	pick({payload = 1, request = {kind = 5}, price = 8}),
	(pick({payload = {}, price = 9}))
)
local v11 = print
local v12 = flag
if count > 1000 then
end
v11(v12, "no")
