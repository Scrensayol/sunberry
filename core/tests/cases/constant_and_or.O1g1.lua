local function show(...)
	print(...)
	return ...
end
for _, v in {false, nil, 1, "x"} do
	for _, v2 in {false, nil, 2} do
		if show(v2, "b") and show(v, "a") then
			print("then", v, v2)
		elseif v2 == 2 then
			print("elseif")
		end
		if (v2 or show(v, "a2")) and (if v then "s" else true) then
			print("then2")
		else
			print("else2")
		end
		while (v2 or v) and show(v, "w") do
			print("loop")
			break
		end
		local v1 = if v2 then show(v, "v") else v and show(v, "v")
		print(v1)
	end
end
