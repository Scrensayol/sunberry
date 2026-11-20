local name = "luna"
local count = 3
local t = {n = 1, list = {}}
print(`hello luna, you have {count} new {if count == 1 then "message" else "messages"}`)
print("literal {braces} and 100% and a ` backtick")
print(`nested inner luna and math {count * 2 + 1}`)
print(`table len {#{1, 2, 3}} first {({10, 20})[1]}`)
print(("luna"):upper(), (`{count}`):rep(2))
local function pair()
	return "a", "b"
end
print(`only first: {pair()}`)
print(("%s and %d"):format("old", 7), ("%*|%*"):format(pair()))
local s = "x"
for i = 1, 3 do
	s ..= i
	s = s .. "-" .. i
	t.n += i
	t.n *= 2
	t.list[i] = i
	local list = t.list
	list[i] ^= 2
	count -= 1
end
local g = 0
local function bump()
	g += 1
	return g
end
local k = "n"
t.n //= 3
t.n %= 7
local v1 = print
local v2 = s
local n = t.n
local v3 = t.list[3]
local v4 = count
bump()
local v5 = g
v1(v2, n, v3, v4, v5, (bump()))
