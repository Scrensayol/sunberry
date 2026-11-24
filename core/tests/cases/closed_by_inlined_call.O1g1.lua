local v1
v1 = select("#", 1)
local v2 = 2
local v3 = 3
local v4 = 4
local v5 = 5
local v6 = 6
local v7 = select("#", 1, 2);
(function()
	print("b", v7, v1, 6)
end)()
local v8 = {}
for _ = 1, 1 do
	for i = 1, 3 do
		local v9 = i * 3
		v8[i] = function(p10)
			v9 += p10
			return i + v9
		end
	end
	print(v8[1](1), v8[1](1), v8[3](2))
end
