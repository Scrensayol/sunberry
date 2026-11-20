local function classify(n)
	if n < 0 then
		return "negative"
	end
	if n == 0 then
		return "zero"
	end
	if n % 2 == 0 then
		return "even"
	end
	return "odd"
end
local function sumUntil(limit)
	local total = 0
	local i = 1
	while total < limit do
		total = total + i
		i = i + 1
	end
	return total, i
end
local function firstPrimeAfter(n2)
	local candidate = n2
	repeat
		candidate = candidate + 1
		local prime = 1 < candidate
		for d = 2, math.floor((math.sqrt(candidate))) do
			if candidate % d == 0 then
				prime = false
				break
			end
		end
	until prime
	return candidate
end
local counts = {}
for _, word in ipairs({
	"a",
	"b",
	"a",
	"c",
	"b",
	"a"
}) do
	if word ~= "c" then
		counts[word] = (counts[word] or 0) + 1
	end
end
for key, value in pairs(counts) do
	print(key, value, (classify(value)))
end
for i2 = 10, 1, -3 do
	print(i2, sumUntil(i2), (firstPrimeAfter(i2)))
end
