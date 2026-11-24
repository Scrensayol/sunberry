local v1 = {force = false, edge = true}
local v3 = {
	get = function(_, p2)
		return {fromEdge = p2 == "edge"}
	end
}
local v4 = {delivery = "https://delivery/", plain = "https://plain/"}
local function resolveHost()
	local v6 = (function()
		if v1.force then
			return v4.delivery
		end
		if v1.edge then
			local v5 = v3:get("edge")
			if v5 and v5.fromEdge then
				return v4.delivery
			end
		end
	end)()
	return v4.plain
end
print(resolveHost())
v1.edge = false
print(resolveHost())
local function pick()
	return ((function()
		if v1.force then
			return v4.delivery
		end
		if v1.edge then
			local v7 = v3:get("edge")
			if v7 and v7.fromEdge then
				return v4.delivery
			end
		end
		return v4.plain
	end)())
end
print(pick())
v1.edge = true
print(pick())
v1.force = true
print(pick())
