local v1 = {flag = false}
local function f(p2, p3)
	local v4 = 0
	while true do
		v4 += 1
		local flag = v1.flag
		local v5 = p2
		if not v5 then
			local flag2 = v1.flag
			v5 = "s"
		end
		if v5 then
			v1.flag = type(p2) == "string"
			print("set", v1.flag)
		end
		if p3 <= v4 then
			break
		end
		return v4
	end
	return v4
end
print(f(nil, 3), f("x", 1), v1.flag)
