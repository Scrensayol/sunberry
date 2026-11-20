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
local v15 = v1 > 3
print(v1, check(v6, "quote"), check(v6, "table"), check(nil, "doc"))
print(pick({price = 7, payload = {kind = 2}}), pick({payload = 1, price = 8, request = {kind = 5}}), pick({price = 9, payload = {}}))
local v16 = print
if v15 then
end
local v17 = {Asset = 1, Bundle = 2, Look = 3}
local v18 = {GamePass = 4, Product = 5}
local v19 = v15
if v1 > 1000 then
	if v15 then
	end
end
v16(v19, "no")
