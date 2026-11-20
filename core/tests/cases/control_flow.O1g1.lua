local function classify(p1)
	if p1 < 0 then
		return "negative"
	end
	if p1 == 0 then
		return "zero"
	end
	if p1 % 2 == 0 then
		return "even"
	end
	return "odd"
end
local function sumUntil(p2)
	local v3 = 0
	local v4 = 1
	while v3 < p2 do
		v3 += v4
		v4 += 1
	end
	return v3, v4
end
local function firstPrimeAfter(p5)
	local v6 = p5
	repeat
		v6 += 1
		local v7 = v6 > 1
		for i = 2, math.floor(math.sqrt(v6)) do
			if v6 % i == 0 then
				v7 = false
				break
			end
		end
	until v7
	return v6
end
local v8 = {}
for _, v in ipairs({
	"a",
	"b",
	"a",
	"c",
	"b",
	"a"
}) do
	if v ~= "c" then
		v8[v] = (v8[v] or 0) + 1
	end
end
for k, v2 in pairs(v8) do
	print(k, v2, classify(v2))
end
for i2 = 10, 1, -3 do
	print(i2, sumUntil(i2), firstPrimeAfter(i2))
end
