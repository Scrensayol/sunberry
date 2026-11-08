-- branches and every loop kind
local function classify(n)
	if n < 0 then
		return "negative"
	elseif n == 0 then
		return "zero"
	elseif n % 2 == 0 then
		return "even"
	else
		return "odd"
	end
end

local function sumUntil(limit)
	local total, i = 0, 1
	while total < limit do
		total += i
		i += 1
	end
	return total, i
end

local function firstPrimeAfter(n)
	local candidate = n
	repeat
		candidate += 1
		local prime = candidate > 1
		for d = 2, math.floor(math.sqrt(candidate)) do
			if candidate % d == 0 then
				prime = false
				break
			end
		end
	until prime
	return candidate
end

local counts = {}
for _, word in ipairs({ "a", "b", "a", "c", "b", "a" }) do
	if word == "c" then
		continue
	end
	counts[word] = (counts[word] or 0) + 1
end

for key, value in pairs(counts) do
	print(key, value, classify(value))
end

for i = 10, 1, -3 do
	print(i, sumUntil(i), firstPrimeAfter(i))
end
