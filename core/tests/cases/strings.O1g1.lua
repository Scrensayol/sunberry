local v1 = "luna"
local v2 = 3
local v3 = {n = 1, list = {}}
print(`hello luna, you have {v2} new {if v2 == 1 then "message" else "messages"}`)
print("literal {braces} and 100% and a ` backtick")
print(`nested inner luna and math {v2 * 2 + 1}`)
print(`table len {#{1, 2, 3}} first {({10, 20})[1]}`)
print(("luna"):upper(), (`{v2}`):rep(2))
local function pair()
	return "a", "b"
end
print(`only first: {pair()}`)
print(("%s and %d"):format("old", 7), ("%*|%*"):format(pair()))
local v4 = "x"
for i = 1, 3 do
	v4 ..= i
	v4 = v4 .. "-" .. i
	v3.n += i
	v3.n *= 2
	v3.list[i] = i
	local list = v3.list
	list[i] ^= 2
	v2 -= 1
end
local v5 = 0
local function bump()
	v5 += 1
	return v5
end
v3.n //= 3
v3.n %= 7
print(v4, v3.n, v3.list[3], v2, bump(), bump())
