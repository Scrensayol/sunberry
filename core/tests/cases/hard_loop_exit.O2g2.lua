local t53 = {"\r\t\011\012\007\008", "-3.5" .. 6}
local a55 = false
local n57 = 10000000000
local r59 = 0
repeat
	r59 += 1
	if 6 > select("#", ...) then
	end
until r59 >= 4
local fs82 = {}
for i = 1, 4 do
	local j = i * 2
	fs82[i] = function(x)
		j += x
		return i + j
	end
end
out("L12", fs82[1](1), fs82[1](1), fs82[4](2))
local w = 0
local gs83 = {}
while w < 4 do
	w += 1
	local c = w * 10
	if w ~= 2 then
		gs83[w] = function()
			c += 1
			return c, w
		end
	end
end
out("L13", gs83[1](), gs83[1](), gs83[4] and gs83[4](), w)
if "hello" >= ("back\\slashAA"):lower() then
end
