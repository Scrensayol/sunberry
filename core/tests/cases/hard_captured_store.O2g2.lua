local function counter()
	local n = 0
	local function get()
		return n
	end
	n = 100
	return get
end
print(counter()())
local fns = {}
for i = 1, 3 do
	local v = i
	fns[i] = function()
		return v
	end
	v = i * 10
end
print(fns[1](), fns[2](), fns[3]())
