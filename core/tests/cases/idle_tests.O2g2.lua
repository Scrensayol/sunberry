local t = {flag = false}
local function f(a, n)
	local i = 0
	while true do
		i += 1
		local flag = t.flag
		local v1 = a
		if not v1 then
			local flag2 = t.flag
			v1 = "s"
		end
		if v1 then
			t.flag = type(a) == "string"
			print("set", t.flag)
		end
		if n <= i then
			break
		end
		return i
	end
	return i
end
print(f(nil, 3), f("x", 1), t.flag)
