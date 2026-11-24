local function show(...)
	print(...)
	return ...
end
for _, a in {false, nil, 1, "x"} do
	for _, b in {false, nil, 2} do
		if show(b, "b") and show(a, "a") then
			print("then", a, b)
		elseif b == 2 then
			print("elseif")
		end
		if (b or show(a, "a2")) and (if a then "s" else true) then
			print("then2")
		else
			print("else2")
		end
		while (b or a) and show(a, "w") do
			print("loop")
			break
		end
		local v = if b then show(a, "v") else a and show(a, "v")
		print(v)
	end
end
