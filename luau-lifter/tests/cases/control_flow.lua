local v1 = {}
local function classify(p2) -- name: classify
	return p2 < 0 and "negative" or (p2 == 0 and "zero" or (p2 % 2 == 0 and "even" or "odd"))
end
local function firstPrimeAfter(p3) -- name: firstPrimeAfter
	while true do
		p3 += 1
		local v4 = p3 > 1
		local sqrt = math.sqrt
		for v5 = 2, math.floor(sqrt(p3)) do
			if p3 % v5 == 0 then
				v4 = false
				break
			end
		end
		if v4 then
			return p3
		end
	end
end
local function sumUntil(p6) -- name: sumUntil
	local v7 = 0
	local v8 = 1
	while v7 < p6 do
		v7 += v8
		v8 += 1
	end
	return v7, v8
end
for _, v9 in ipairs({
	"a",
	"b",
	"a",
	"c",
	"b",
	"a"
}) do
	if v9 == "c" then
		continue
	end
	v1[v9] = (v1[v9] or 0) + 1
end
for v10, v11 in pairs(v1) do
	print(v10, v11, classify(v11))
end
for v12 = 10, 1, -3 do
	print(v12, sumUntil(v12), firstPrimeAfter(v12))
end