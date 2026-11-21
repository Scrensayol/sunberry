-- a friend's case: a branch constant folding can prove dead (O1+ drops it entirely), with a
-- numeric for over nils inside that would error if it ever ran
if false and ... then
	for i = nil, nil do
		return
	end
end
print("alive")
