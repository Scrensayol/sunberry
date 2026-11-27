-- hard tier, x3: a store into a local that a closure has already captured, which nothing in
-- the function reads again (Rosetta's loop_closures, shrunk). The normalizer dropped it as a
-- dead load, and the closure read the value from before it
local function counter()
	local n = 0
	local function get()
		return n
	end
	n = 100
	return get
end
print(counter()())

-- the same, closed by CLOSEUPVALS at the end of each loop iteration
local fns = {}
for i = 1, 3 do
	local v = i
	fns[i] = function()
		return v
	end
	v = i * 10
end
print(fns[1](), fns[2](), fns[3]())
