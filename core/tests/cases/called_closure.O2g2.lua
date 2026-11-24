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
	local _ = (function()
		if flags.force then
			return urls.delivery
		end
		if flags.edge then
			local data = layers:get("edge")
			if data and data.fromEdge then
				return urls.delivery
			end
		end
	end)()
	return urls.plain
end
local v1 = print
if flags.force then
	local delivery = urls.delivery
else
	local v2
	local edge = flags.edge
	if edge then
		v2 = layers:get("edge")
		edge = v2
	end
	local v3 = not edge
	if not v3 then
		local v4 = not v2.fromEdge
		if not v4 then
			local delivery2 = urls.delivery
			v4 = false
		end
		v3 = v4
	end
end
v1(urls.plain)
flags.edge = false
local v5 = print
if flags.force then
	local delivery3 = urls.delivery
else
	local v6
	local edge2 = flags.edge
	if edge2 then
		v6 = layers:get("edge")
		edge2 = v6
	end
	local v7 = not edge2
	if not v7 then
		local v8 = not v6.fromEdge
		if not v8 then
			local delivery4 = urls.delivery
			v8 = false
		end
		v7 = v8
	end
end
v5(urls.plain)
local function pick()
	local host = (function()
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
	end)()
	return host
end
local v9 = print
local delivery5
if flags.force then
	delivery5 = urls.delivery
else
	local v10
	local edge3 = flags.edge
	if edge3 then
		v10 = layers:get("edge")
		edge3 = v10
	end
	local v11 = not edge3
	if not v11 then
		local v12 = not v10.fromEdge
		if not v12 then
			delivery5 = urls.delivery
			v12 = false
		end
		v11 = v12
	end
	if v11 then
		delivery5 = urls.plain
	end
end
v9(delivery5)
flags.edge = true
local v13 = print
local delivery6
if flags.force then
	delivery6 = urls.delivery
else
	local v14
	local edge4 = flags.edge
	if edge4 then
		v14 = layers:get("edge")
		edge4 = v14
	end
	local v15 = not edge4
	if not v15 then
		local v16 = not v14.fromEdge
		if not v16 then
			delivery6 = urls.delivery
			v16 = false
		end
		v15 = v16
	end
	if v15 then
		delivery6 = urls.plain
	end
end
v13(delivery6)
flags.force = true
local v17 = print
local delivery7
if flags.force then
	delivery7 = urls.delivery
else
	local v18
	local edge5 = flags.edge
	if edge5 then
		v18 = layers:get("edge")
		edge5 = v18
	end
	local v19 = not edge5
	if not v19 then
		local v20 = not v18.fromEdge
		if not v20 then
			delivery7 = urls.delivery
			v20 = false
		end
		v19 = v20
	end
	if v19 then
		delivery7 = urls.plain
	end
end
v17(delivery7)
