-- `continue` is only a keyword where it can't be a variable: `while continue do` reads one
local out = {}

-- a friend's case: never runs (the global is nil), but has to come back the same
while continue do
	for i = 1, 9e9, 1 do
		continue
	end
	continue
end
while continue do
	for i = 1, 9e9, 1 do
		break
	end
	break
end

-- the same shapes, running
continue = 3
while continue > 0 do
	for i = 1, 3 do
		if i == 2 then
			continue
		end
		table.insert(out, i)
	end
	continue -= 1
	continue
end
continue = true
while continue do
	for i = 1, 9e9, 1 do
		table.insert(out, "in")
		break
	end
	break
end

-- as a local, a field and a function
local function type(continue)
	return continue .. "!"
end
local t = {continue = 1}
t.continue += 1
local continue = type("go")
table.insert(out, continue)
table.insert(out, t.continue)
print(table.concat(out, " "))
