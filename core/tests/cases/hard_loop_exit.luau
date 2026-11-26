-- hard tier, x2: a repeat loop's exit has to come after it, not inside: closures made in the
-- loops after it each get their own variable (Rosetta's seed 1063, shrunk)
do
	local t53 = {(if (type(0) == "string") then string.rep("\255\128", 3) else "\r\t\v\f\a\b"), ("-3.5" .. (#("\r\t\v\f\a\b")))}
	local a55 = not true
	local n57 = 1e10
	local r59 = 0
	repeat
		r59 += 1
		if not (string.len('single') > select("#", ...)) then
		end
	until r59 >= 4
end
do
	local fs82 = {}
for i = 1, 4 do
	local j = i * 2
	fs82[i] = function(x) j += x return i + j end
end
out("L12", fs82[1](1), fs82[1](1), fs82[4](2))
local gs83, w = {}, 0
while w < 4 do
	w += 1
	local c = w * 10
	if w == 2 then continue end
	gs83[w] = function() c += 1 return c, w end
end
out("L13", gs83[1](), gs83[1](), gs83[4] and gs83[4](), w)
	if (((nil and true) or (true and true or 0)) or (math.huge or not "s")) then
		if ("hello" >= ("back\\slash" .. "\x41\65"):lower()) then
		end
	end
end
