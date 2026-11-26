local v1 = {
	if type(0) == "string" then string.rep("\255\128", 3) else "\r\t\011\012\007\008",
	"-3.5" .. 6
}
local v2 = 0
repeat
	v2 += 1
	if string.len("single") > select("#", ...) then
	end
until v2 >= 4
local v3 = {}
for i = 1, 4 do
	local v4 = i * 2
	v3[i] = function(p5)
		v4 += p5
		return i + v4
	end
end
out("L12", v3[1](1), v3[1](1), v3[4](2))
local v6 = {}
local v7 = 0
while v7 < 4 do
	v7 += 1
	local v8 = v7 * 10
	if v7 ~= 2 then
		v6[v7] = function()
			v8 += 1
			return v8, v7
		end
	end
end
out("L13", v6[1](), v6[1](), v6[4] and v6[4](), v7)
if "hello" >= ("back\\slashAA"):lower() then
end
