-- `(function() ... end)()` is inlined at -O2, leaving a copy with early returns and a statement
-- between them that only reads as that closure called again; its result goes nowhere here
local flags = { force = false, edge = true }
local layers = { get = function(_, key) return { fromEdge = key == "edge" } end }
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
print(resolveHost())
flags.edge = false
print(resolveHost())

-- and with the result used
local function pick()
	local host = (function()
		if flags.force then
			return urls.delivery
		end
		if flags.edge then
			local data = layers:get("edge")
			if data and data.fromEdge then
				return urls.delivery
			end
		end
		return urls.plain
	end)()
	return host
end
print(pick())
flags.edge = true
print(pick())
flags.force = true
print(pick())
