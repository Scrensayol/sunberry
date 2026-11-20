local v1 = {}
v1.__index = v1
function v1.new(p2, p3)
	local self = setmetatable({}, v1)
	self.owner = p2
	self.balance = p3 or 0
	self.history = {}
	return self
end
function v1:deposit(p4)
	if p4 <= 0 then
		error("deposit must be positive")
	end
	self.balance += p4
	table.insert(self.history, {kind = "deposit", amount = p4})
end
function v1:withdraw(p5)
	if self.balance < p5 then
		return false, "insufficient funds"
	end
	self.balance -= p5
	table.insert(self.history, {kind = "withdraw", amount = p5})
	return true
end
function v1:statement()
	local v6 = {}
	for i, v in ipairs(self.history) do
		v6[i] = ("%s %d"):format(v.kind, v.amount)
	end
	return table.concat(v6, "\n")
end
local v7 = v1.new("sam", 100)
v7:deposit(50)
local v8, v9 = v7:withdraw(500)
if not v8 then
	print(v9)
end
print(v7:statement())
