local function counter()
	local v1 = 0
	local function get()
		return v1
	end
	v1 = 100
	return get
end
print(counter()())
local v2 = {}
for i = 1, 3 do
	local v3 = i
	v2[i] = function()
		return v3
	end
	v3 = i * 10
end
print(v2[1](), v2[2](), v2[3]())
