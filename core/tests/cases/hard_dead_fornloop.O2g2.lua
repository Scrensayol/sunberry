local out = {}
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
	table.insert(out, 1)
	table.insert(out, 3)
	continue -= 1
end
continue = true
while continue do
	for _ = 1, 9000000000 do
		table.insert(out, "in")
		break
	end
	break
end
local function type(continue2)
	return continue2 .. "!"
end
local t = {continue = 1}
t.continue += 1
local continue3 = "go!"
table.insert(out, continue3)
table.insert(out, t.continue)
print(table.concat(out, " "))
