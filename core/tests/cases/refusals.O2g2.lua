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
local flag = count > 3
local v9 = print
local v10 = count
local v11
local v12 = tree ~= nil
if v12 then
	local node3 = tree
	local v13 = false
	local v14
	while node3 ~= nil do
		if node3.kind == "quote" then
			v14 = true
			v13 = true
			break
		end
		node3 = node3.parent
	end
	if not v13 then
		v14 = false
	end
	v12 = v14
end
local v15 = not v12
if not v15 then
	v11 = "inside quote"
	v15 = false
end
if v15 then
	v11 = "outside quote"
end
local v16
local v17 = tree ~= nil
if v17 then
	local node4 = tree
	local v18 = false
	local v19
	while node4 ~= nil do
		if node4.kind == "table" then
			v19 = true
			v18 = true
			break
		end
		node4 = node4.parent
	end
	if not v18 then
		v19 = false
	end
	v17 = v19
end
local v20 = not v17
if not v20 then
	v16 = "inside table"
	v20 = false
end
if v20 then
	v16 = "outside table"
end
local v21 = false
v9(v10, v11, v16, "outside doc")
local v22 = print
local state2 = {price = 7, payload = {kind = 2}}
local v23
if state2 == nil then
	v23 = nil
else
	local payload3 = state2.payload
	local requestType3 = state2.request and state2.request.kind
	local kind5
	local v24
	local v25 = type(payload3) == "table"
	if v25 then
		kind5 = payload3.kind
		v25 = kind5 == A.Asset or kind5 == A.Bundle or kind5 == A.Look
	end
	local v26 = not v25
	if not v26 then
		v24 = true
		v26 = false
	end
	local v27 = v26
	if v27 then
		local v28 = not (requestType3 == B.GamePass or requestType3 == B.Product)
		if not v28 then
			v24 = true
			v28 = false
		end
		v27 = v28
	end
	if v27 then
		v24 = false
	end
	v23 = if v24 then state2.price else nil
end
local state3 = {payload = 1, price = 8, request = {kind = 5}}
local v29
if state3 == nil then
	v29 = nil
else
	local payload4 = state3.payload
	local requestType4 = state3.request and state3.request.kind
	local kind6
	local v30
	local v31 = type(payload4) == "table"
	if v31 then
		kind6 = payload4.kind
		v31 = kind6 == A.Asset or kind6 == A.Bundle or kind6 == A.Look
	end
	local v32 = not v31
	if not v32 then
		v30 = true
		v32 = false
	end
	local v33 = v32
	if v33 then
		local v34 = not (requestType4 == B.GamePass or requestType4 == B.Product)
		if not v34 then
			v30 = true
			v34 = false
		end
		v33 = v34
	end
	if v33 then
		v30 = false
	end
	v29 = if v30 then state3.price else nil
end
local state4 = {price = 9, payload = {}}
local v35
if state4 == nil then
	v35 = nil
else
	local payload5 = state4.payload
	local requestType5 = state4.request and state4.request.kind
	local kind7
	local v36
	local v37 = type(payload5) == "table"
	if v37 then
		kind7 = payload5.kind
		v37 = kind7 == A.Asset or kind7 == A.Bundle or kind7 == A.Look
	end
	local v38 = not v37
	if not v38 then
		v36 = true
		v38 = false
	end
	local v39 = v38
	if v39 then
		local v40 = not (requestType5 == B.GamePass or requestType5 == B.Product)
		if not v40 then
			v36 = true
			v40 = false
		end
		v39 = v40
	end
	if v39 then
		v36 = false
	end
	v35 = if v36 then state4.price else nil
end
v22(v23, v29, v35)
local v41 = print
if flag then
end
local v42 = flag
if count > 1000 then
	if flag then
	end
end
v41(v42, "no")
