local v1 = {}
v1.__index = v1
function v1.new(v2, v3)
	local v4 = setmetatable({}, v1)
	v4.owner = v2
	v4.balance = v3 or 0
	v4.history = {}
	return v4
end
function v1:deposit(v5)
	if v5 <= 0 then
		error("deposit must be positive")
	end
	self.balance = self.balance + v5
	table.insert(self.history, {kind = "deposit", amount = v5})
end
function v1:withdraw(v6)
	if self.balance < v6 then
		return false, "insufficient funds"
	end
	self.balance = self.balance - v6
	table.insert(self.history, {kind = "withdraw", amount = v6})
	return true
end
function v1:statement()
	local v7 = {}
	for i, v in ipairs(self.history) do
		v7[i] = ("%s %d"):format(v.kind, v.amount)
	end
	return table.concat(v7, "\n")
end
local v8 = v1.new("sam", 100)
v8:deposit(50)
local v9, v10 = v8:withdraw(500)
if not v9 then
	print(v10)
end
print(v8:statement())
