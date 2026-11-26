local calls = 0
local function bound(v)
	if __rosetta then
		return v
	end
	calls += 1
	return v
end
local v1 = __rosetta
calls = if v1 then calls else calls + 1
local v2 = 1
local v3 = __rosetta
calls = if v3 then calls else calls + 1
local v4 = 3
local v5 = __rosetta
calls = if v5 then calls else calls + 1
for i = v2, v4 do
	print("bounded", i)
end
print("bound calls", calls)
