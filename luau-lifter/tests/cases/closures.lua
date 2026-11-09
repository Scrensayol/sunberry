local v4, v5 = (function(p_u_1, p_u_2) -- name: makeCounter
	local v_u_3 = p_u_1
	return function()
		-- upvalues: (ref) v_u_3, (copy) p_u_2
		v_u_3 += p_u_2
		return v_u_3
	end, function()
		-- upvalues: (ref) v_u_3, (copy) p_u_1
		v_u_3 = p_u_1
	end
end)(10, 5)
print(v4(), v4())
v5()
print(v4())
local v_u_6 = nil
v_u_6 = (function(p_u_7) -- name: memoize
	local v_u_8 = {}
	return function(p9)
		-- upvalues: (copy) v_u_8, (copy) p_u_7
		local v10 = v_u_8[p9]
		if v10 == nil then
			v10 = p_u_7(p9)
			v_u_8[p9] = v10
		end
		return v10
	end
end)(function(p11)
	-- upvalues: (ref) v_u_6
	if p11 < 2 then
		return p11
	else
		return v_u_6(p11 - 1) + v_u_6(p11 - 2)
	end
end)
print(v_u_6(40))
local v12 = {}
for v_u_13 = 1, 3 do
	v12[v_u_13] = function(p14)
		-- upvalues: (copy) v_u_13
		return p14 * v_u_13
	end
end
print(v12[2](21))