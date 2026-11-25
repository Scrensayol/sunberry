local function bump()
	(function(_)
		if skip then
			return
		end
		return 0
	end)(1)
end
print(bump(), bump())
for _ = 1, 1 do
	if not skip then
		print("loop", bump(), bump())
	end
end
