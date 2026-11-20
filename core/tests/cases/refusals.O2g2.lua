local count = 0
local tick
function tick(n)
	count += 1
	if n > 0 then
		tick(n - 1)
	end
end
local v1 = tick
v1(3)
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
	if found then
		local node2 = start
		local v2 = false
		local v3
		while node2 ~= nil do
			if node2.kind == kind2 then
				v3 = true
				v2 = true
				break
			end
			node2 = node2.parent
		end
		if not v2 then
			v3 = false
		end
		if v3 then
			return "inside " .. kind2
		end
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
	local v4
	local v5 = type(payload2) == "table"
	if v5 then
		kind4 = payload2.kind
		v5 = kind4 == A.Asset or kind4 == A.Bundle or kind4 == A.Look
	end
	local v6 = not v5
	if not v6 then
		v4 = true
		v6 = false
	end
	local v7 = v6
	if v7 then
		local v8 = not (requestType2 == B.GamePass or requestType2 == B.Product)
		if not v8 then
			v4 = true
			v8 = false
		end
		v7 = v8
	end
	if v7 then
		v4 = false
	end
	if not v4 then
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
	local v9 = {}
	local function IsA(_, class)
		return class == "FloatCurve"
	end
	v9.IsA = IsA
	local function GetKeys()
		return keys
	end
	v9.GetKeys = GetKeys
	return v9
end
local function hasKeys(folder)
	local X = folder.X
	local Y = folder.Y
	local Z = folder.Z
	local v10 = if X then X:IsA("FloatCurve") and #X:GetKeys() > 0 else false
	if not v10 then
		v10 = if Y then Y:IsA("FloatCurve") and #Y:GetKeys() > 0 else false
		if not v10 then
			if Z then
				v10 = Z:IsA("FloatCurve")
				if v10 then
					v10 = #Z:GetKeys() > 0
				end
				return v10
			end
			v10 = false
		end
	end
	return v10
end
print(hasKeys({X = curve({}), Z = curve({1})}), hasKeys({}), (hasKeys({Y = curve({1, 2})})))
local flag = count > 3
local v11 = print
local v12 = count
local v13
local v14 = tree ~= nil
if v14 then
	local node3 = tree
	local v15 = false
	local v16
	while node3 ~= nil do
		if node3.kind == "quote" then
			v16 = true
			v15 = true
			break
		end
		node3 = node3.parent
	end
	if not v15 then
		v16 = false
	end
	v14 = v16
end
local v17 = not v14
if not v17 then
	v13 = "inside quote"
	v17 = false
end
if v17 then
	v13 = "outside quote"
end
local v18
local v19 = tree ~= nil
if v19 then
	local node4 = tree
	local v20 = false
	local v21
	while node4 ~= nil do
		if node4.kind == "table" then
			v21 = true
			v20 = true
			break
		end
		node4 = node4.parent
	end
	if not v20 then
		v21 = false
	end
	v19 = v21
end
local v22 = not v19
if not v22 then
	v18 = "inside table"
	v22 = false
end
if v22 then
	v18 = "outside table"
end
local v23 = false
v11(v12, v13, v18, "outside doc")
local v24 = print
local state2 = {price = 7, payload = {kind = 2}}
local v25
if state2 == nil then
	v25 = nil
else
	local payload3 = state2.payload
	local requestType3 = state2.request and state2.request.kind
	local kind5
	local v26
	local v27 = type(payload3) == "table"
	if v27 then
		kind5 = payload3.kind
		v27 = kind5 == A.Asset or kind5 == A.Bundle or kind5 == A.Look
	end
	local v28 = not v27
	if not v28 then
		v26 = true
		v28 = false
	end
	local v29 = v28
	if v29 then
		local v30 = not (requestType3 == B.GamePass or requestType3 == B.Product)
		if not v30 then
			v26 = true
			v30 = false
		end
		v29 = v30
	end
	if v29 then
		v26 = false
	end
	v25 = if v26 then state2.price else nil
end
local state3 = {payload = 1, price = 8, request = {kind = 5}}
local v31
if state3 == nil then
	v31 = nil
else
	local payload4 = state3.payload
	local requestType4 = state3.request and state3.request.kind
	local kind6
	local v32
	local v33 = type(payload4) == "table"
	if v33 then
		kind6 = payload4.kind
		v33 = kind6 == A.Asset or kind6 == A.Bundle or kind6 == A.Look
	end
	local v34 = not v33
	if not v34 then
		v32 = true
		v34 = false
	end
	local v35 = v34
	if v35 then
		local v36 = not (requestType4 == B.GamePass or requestType4 == B.Product)
		if not v36 then
			v32 = true
			v36 = false
		end
		v35 = v36
	end
	if v35 then
		v32 = false
	end
	v31 = if v32 then state3.price else nil
end
local state4 = {price = 9, payload = {}}
local v37
if state4 == nil then
	v37 = nil
else
	local payload5 = state4.payload
	local requestType5 = state4.request and state4.request.kind
	local kind7
	local v38
	local v39 = type(payload5) == "table"
	if v39 then
		kind7 = payload5.kind
		v39 = kind7 == A.Asset or kind7 == A.Bundle or kind7 == A.Look
	end
	local v40 = not v39
	if not v40 then
		v38 = true
		v40 = false
	end
	local v41 = v40
	if v41 then
		local v42 = not (requestType5 == B.GamePass or requestType5 == B.Product)
		if not v42 then
			v38 = true
			v42 = false
		end
		v41 = v42
	end
	if v41 then
		v38 = false
	end
	v37 = if v38 then state4.price else nil
end
v24(v25, v31, v37)
local v43 = print
if flag then
end
local v44 = flag
if count > 1000 then
	if flag then
	end
end
v43(v44, "no")
