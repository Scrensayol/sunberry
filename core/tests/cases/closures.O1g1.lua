local v4, v5 = (function(v1, v2)
	local v3 = v1
	return function()
		v3 += v2
		return v3
	end, function()
		v3 = v1
	end
end)(10, 5)
print(v4(), v4())
v5()
print(v4())
local function memoize(v6)
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
local v10
v10 = memoize(function(v11)
	if v11 < 2 then
		return v11
	end
	return v10(v11 - 1) + v10(v11 - 2)
end)
local v12 = print
local v13 = v10
v12(v13(40))
local v14 = {}
for i = 1, 3 do
	v14[i] = function(v15)
		return v15 * i
	end
end
print(v14[2](21))
