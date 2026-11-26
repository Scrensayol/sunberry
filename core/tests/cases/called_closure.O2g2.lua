local flags = {force = false, edge = true}
local layers = {
	get = function(_, key)
		return {fromEdge = key == "edge"}
	end
}
local urls = {}
urls.delivery = "https://delivery/"
urls.plain = "https://plain/"
local function resolveHost()
	local function v1()
		if flags.force then
			return urls.delivery
		end
		if flags.edge then
			local data = layers:get("edge")
			if data and data.fromEdge then
				return urls.delivery
			end
		end
	end
	local _ = v1()
	return urls.plain
end
local v2 = print
if flags.force then
	local delivery = urls.delivery
else
	local v3
	local edge = flags.edge
	if edge then
		v3 = layers:get("edge")
		edge = v3
	end
	local v4 = not edge
	if not v4 then
		local v5 = not v3.fromEdge
		if not v5 then
			local delivery2 = urls.delivery
			v5 = false
		end
		v4 = v5
	end
end
v2(urls.plain)
flags.edge = false
local v6 = print
if flags.force then
	local delivery3 = urls.delivery
else
	local v7
	local edge2 = flags.edge
	if edge2 then
		v7 = layers:get("edge")
		edge2 = v7
	end
	local v8 = not edge2
	if not v8 then
		local v9 = not v7.fromEdge
		if not v9 then
			local delivery4 = urls.delivery
			v9 = false
		end
		v8 = v9
	end
end
v6(urls.plain)
local function pick()
	local function v10()
		if flags.force then
			return urls.delivery
		end
		if flags.edge then
			local data2 = layers:get("edge")
			if data2 and data2.fromEdge then
				return urls.delivery
			end
		end
		return urls.plain
	end
	local host = v10()
	return host
end
local v11 = print
local delivery5
if flags.force then
	delivery5 = urls.delivery
else
	local v12
	local edge3 = flags.edge
	if edge3 then
		v12 = layers:get("edge")
		edge3 = v12
	end
	local v13 = not edge3
	if not v13 then
		local v14 = not v12.fromEdge
		if not v14 then
			delivery5 = urls.delivery
			v14 = false
		end
		v13 = v14
	end
	if v13 then
		delivery5 = urls.plain
	end
end
v11(delivery5)
flags.edge = true
local v15 = print
local delivery6
if flags.force then
	delivery6 = urls.delivery
else
	local v16
	local edge4 = flags.edge
	if edge4 then
		v16 = layers:get("edge")
		edge4 = v16
	end
	local v17 = not edge4
	if not v17 then
		local v18 = not v16.fromEdge
		if not v18 then
			delivery6 = urls.delivery
			v18 = false
		end
		v17 = v18
	end
	if v17 then
		delivery6 = urls.plain
	end
end
v15(delivery6)
flags.force = true
local v19 = print
local delivery7
if flags.force then
	delivery7 = urls.delivery
else
	local v20
	local edge5 = flags.edge
	if edge5 then
		v20 = layers:get("edge")
		edge5 = v20
	end
	local v21 = not edge5
	if not v21 then
		local v22 = not v20.fromEdge
		if not v22 then
			delivery7 = urls.delivery
			v22 = false
		end
		v21 = v22
	end
	if v21 then
		delivery7 = urls.plain
	end
end
v19(delivery7)
