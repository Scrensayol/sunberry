local v4, v5 = (function(v1, v2)
	local v3 = v1
	return function()
		v3 = v3 + v2
		return v3
	end, function()
		v3 = v1
	end
end)(10, 5)
print(v4(), v4())
v5()
print(v4())
local function v10(v6)
	local v7 = {}
	return function(v8)
		local v9 = v7[v8]
		if v9 == nil then
			v9 = v6(v8)
			v7[v8] = v9
		end
		return v9
	end
end
local v11
v11 = v10(function(v12)
	if v12 < 2 then
		return v12
	end
	return v11(v12 - 1) + v11(v12 - 2)
end)
local v13 = print
local v14 = v11
v13(v14(40))
local v15 = {}
for i = 1, 3 do
	v15[i] = function(v16)
		return v16 * i
	end
end
print(v15[2](21))
