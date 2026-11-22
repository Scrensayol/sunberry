local function makeCounter(start, step)
	local value = start
	return function()
		value += step
		return value
	end, function()
		value = start
	end
end
local value2 = 10
local v1
v1 = 5
local function v2()
	value2 += v1
	return value2
end
local v3
v3 = 10
local function reset()
	value2 = v3
end
print(v2(), v2())
reset()
print(v2())
local function memoize(fn)
	local cache = {}
	return function(n)
		local hit = cache[n]
		if hit == nil then
			hit = fn(n)
			cache[n] = hit
		end
		return hit
	end
end
local fib
fib = memoize(function(n2)
	if n2 < 2 then
		return n2
	end
	return fib(n2 - 1) + fib(n2 - 2)
end)
print(fib(40))
local handlers = {}
for i = 1, 3 do
	handlers[i] = function(x)
		return x * i
	end
end
print(handlers[2](21))
