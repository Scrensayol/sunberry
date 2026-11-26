local v1 = {force = false, edge = true}
local v3 = {
	get = function(_, p2)
		return {fromEdge = p2 == "edge"}
	end
}
local v4 = {delivery = "https://delivery/", plain = "https://plain/"}
local function resolveHost()
	local function v6()
		if v1.force then
			return v4.delivery
		end
		if v1.edge then
			local v5 = v3:get("edge")
			if v5 and v5.fromEdge then
				return v4.delivery
			end
		end
	end
	local v7 = v6()
	return v4.plain
end
print(resolveHost())
v1.edge = false
print(resolveHost())
local function pick()
	local function v9()
		if v1.force then
			return v4.delivery
		end
		if v1.edge then
			local v8 = v3:get("edge")
			if v8 and v8.fromEdge then
				return v4.delivery
			end
		end
		return v4.plain
	end
	return (v9())
end
print(pick())
v1.edge = true
print(pick())
v1.force = true
print(pick())
