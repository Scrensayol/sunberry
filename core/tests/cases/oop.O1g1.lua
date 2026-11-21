local v1 = {}
v1.__index = v1
function v1.new(owner, p2)
	local self = setmetatable({}, v1)
	self.owner = owner
	self.balance = p2 or 0
	self.history = {}
	return self
end
function v1:deposit(amount)
	if amount <= 0 then
		error("deposit must be positive")
	end
	self.balance += amount
	table.insert(self.history, {kind = "deposit", amount = amount})
end
function v1:withdraw(amount2)
	if self.balance < amount2 then
		return false, "insufficient funds"
	end
	self.balance -= amount2
	table.insert(self.history, {kind = "withdraw", amount = amount2})
	return true
end
function v1:statement()
	local v3 = {}
	for i, v in ipairs(self.history) do
		v3[i] = ("%s %d"):format(v.kind, v.amount)
	end
	return table.concat(v3, "\n")
end
local v4 = v1.new("sam", 100)
v4:deposit(50)
local v5, v6 = v4:withdraw(500)
if not v5 then
	print(v6)
end
print(v4:statement())
