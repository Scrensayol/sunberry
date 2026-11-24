-- `x and <a constant that is true> or y` inside a condition compiles to a jump straight on
-- to the condition's next part, with the `or y` side left behind it unreachable
local function show(...) print(...) return ... end
for _, a in {false, nil, 1, "x"} do
	for _, b in {false, nil, 2} do
		if (show(b, "b") and (0 / 0) or nil) and show(a, "a") then
			print("then", a, b)
		elseif b == 2 then
			print("elseif")
		end
		if ((b and 1 or show(a, "a2")) and (if a then ("s" or "s") else true)) then
			print("then2")
		else
			print("else2")
		end
		while (b and "t" or a) and show(a, "w") do
			print("loop")
			break
		end
		local v = (b and 1 or a) and show(a, "v")
		print(v)
	end
end
