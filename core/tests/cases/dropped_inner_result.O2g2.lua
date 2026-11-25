local zero
zero = 0
local function bump()
	local v1
	v1 = 0
	local function inner(_)
		if skip then
			return
		end
		return 0
	end
	inner()
end
local v2 = print
local function inner2(_)
	if skip then
		return
	end
	return 0
end
if skip and false then
end
v2(nil, bump())
if skip then
	return
end
local v3 = print
local v4 = "loop"
local function inner3(_)
	if skip then
		return
	end
	return 0
end
if skip and false then
end
v3(v4, nil, bump())
