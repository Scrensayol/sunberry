-- shapes the new core used to refuse, mostly what -O2 inlining leaves behind

-- a local declared first, then a function that calls itself through it
local count = 0
local tick
tick = function(n)
	count += 1
	if n > 0 then
		tick(n - 1)
	end
end
tick(3)

-- a helper with a loop, inlined into a condition
local function isInside(node, kind)
	while node ~= nil do
		if node.kind == kind then
			return true
		end
		node = node.parent
	end
	return false
end
local tree = { kind = "leaf", parent = { kind = "quote", parent = { kind = "doc" } } }
local function check(start, kind)
	local found = start ~= nil
	if found and isInside(start, kind) then
		return "inside " .. kind
	end
	return "outside " .. kind
end

-- a helper with early returns and a local the next tests compare against
local A = { Asset = 1, Bundle = 2, Look = 3 }
local B = { GamePass = 4, Product = 5 }
local function isSupported(payload, requestType)
	if type(payload) == "table" then
		local kind = payload.kind
		if kind == A.Asset or kind == A.Bundle or kind == A.Look then
			return true
		end
	end
	if requestType == B.GamePass or requestType == B.Product then
		return true
	end
	return false
end
local function pick(state)
	if state == nil then
		return nil
	end
	if not isSupported(state.payload, state.request and state.request.kind) then
		return nil
	end
	return state.price
end

-- a loop whose end is the end of the function, through an if
local function scan(list, limit)
	if #list > 0 then
		local i = 0
		repeat
			i += 1
			if list[i] == limit then
				count += 100
				break
			end
		until i >= #list
	elseif limit then
		count += 1
	end
end
scan({ 1, 2, 3 }, 2)
scan({}, true)

-- an `or` chain of method calls on locals, each part a boolean
local function curve(keys)
	return { IsA = function(_, class) return class == "FloatCurve" end, GetKeys = function() return keys end }
end
local function hasKeys(folder)
	local X = folder.X
	local Y = folder.Y
	local Z = folder.Z
	return (if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false)
		or (if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false)
		or (if Z then Z:IsA("FloatCurve") and #Z:GetKeys() > 0 else false)
end
print(hasKeys({ X = curve({}), Z = curve({ 1 }) }), hasKeys({}), hasKeys({ Y = curve({ 1, 2 }) }))

local flag = count > 3
print(count, check(tree, "quote"), check(tree, "table"), check(nil, "doc"))
print(pick({ payload = { kind = 2 }, price = 7 }), pick({ payload = 1, request = { kind = 5 }, price = 8 }), pick({ payload = {}, price = 9 }))
print((flag and nil) or flag, (count > 1000 and (flag and nil)) or "no")
