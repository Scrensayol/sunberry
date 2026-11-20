-- interpolation and compound assignment edge cases
local name, count = "luna", 3
local t = { n = 1, list = {} }

print(`hello {name}, you have {count} new {count == 1 and "message" or "messages"}`)
print(`literal \{braces} and 100% and a \` backtick`)
print(`nested {`inner {name}`} and math {count * 2 + 1}`)
print(`table len {#{ 1, 2, 3 }} first {({ 10, 20 })[1]}`)
print((`{name}`):upper(), (`{count}`):rep(2))
local function pair()
	return "a", "b"
end
print(`only first: {pair()}`)
print(("%s and %d"):format("old", 7), ("%*|%*"):format(pair()))

local s = "x"
for i = 1, 3 do
	s ..= i
	s ..= "-" .. i
	t.n += i
	t.n *= 2
	t.list[i] = i
	t.list[i] ^= 2
	count -= 1
end
local g = 0
local function bump()
	g += 1
	return g
end
local k = "n"
t[k] //= 3
t[k] %= 7
print(s, t.n, t.list[3], count, bump(), bump())
