-- upvalues, nested closures and recursion
local function makeCounter(start, step)
	local value = start
	return function()
		value += step
		return value
	end, function()
		value = start
	end
end

local next, reset = makeCounter(10, 5)
print(next(), next())
reset()
print(next())

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
fib = memoize(function(n)
	if n < 2 then
		return n
	end
	return fib(n - 1) + fib(n - 2)
end)
print(fib(40))

local handlers = {}
for i = 1, 3 do
	handlers[i] = function(x)
		return x * i
	end
end
print(handlers[2](21))
