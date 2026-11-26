local v1 = {}
while continue do
	for _ = 1, 9000000000 do
	end
end
while continue do
	for _ = 1, 9000000000 do
		break
	end
	break
end
continue = 3
while continue > 0 do
	for i = 1, 3 do
		if i ~= 2 then
			table.insert(v1, i)
		end
	end
	continue -= 1
end
continue = true
while continue do
	for _ = 1, 9000000000 do
		table.insert(v1, "in")
		break
	end
	break
end
local function type(p2)
	return p2 .. "!"
end
local v3 = {continue = 1}
v3.continue += 1
local v4 = type("go")
table.insert(v1, v4)
table.insert(v1, v3.continue)
print(table.concat(v1, " "))
