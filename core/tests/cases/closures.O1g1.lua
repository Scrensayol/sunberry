local v4, v5 = (function(p1, p2)
	local v3 = p1
	return function()
		v3 += p2
		return v3
	end, function()
		v3 = p1
	end
end)(10, 5)
print(v4(), v4())
v5()
print(v4())
local function memoize(p6)
	local v7 = {}
	return function(p8)
		local v9 = v7[p8]
		if v9 == nil then
			v9 = p6(p8)
			v7[p8] = v9
		end
		return v9
	end
end
local v10
v10 = memoize(function(p11)
	if p11 < 2 then
		return p11
	end
	return v10(p11 - 1) + v10(p11 - 2)
end)
print(v10(40))
local v12 = {}
for i = 1, 3 do
	v12[i] = function(p13)
		return p13 * i
	end
end
print(v12[2](21))
