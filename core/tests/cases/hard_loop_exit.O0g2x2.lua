local v1 = {
	if type(0) == "string" then string.rep("\255\128", 3) else "\r\t\011\012\007\008",
	"-3.5" .. #"\r\t\011\012\007\008"
}
local v2 = not true
local v3 = 0
repeat
	v3 += 1
	if string.len("single") > select("#", ...) then
	end
until v3 >= 4
local v4 = {}
for i = 1, 4 do
	local v5 = i * 2
	v4[i] = function(p6)
		v5 += p6
		return i + v5
	end
end
out("L12", v4[1](1), v4[1](1), v4[4](2))
local v7 = {}
local v8 = 0
while v8 < 4 do
	v8 += 1
	local v9 = v8 * 10
	if v8 ~= 2 then
		v7[v8] = function()
			v9 += 1
			return v9, v8
		end
	end
end
out("L13", v7[1](), v7[1](), v7[4] and v7[4](), v8)
if "hello" >= ("back\\slash" .. "AA"):lower() then
end
