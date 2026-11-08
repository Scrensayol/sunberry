-- metatable classes with methods
local Account = {}
Account.__index = Account

function Account.new(owner, balance)
	local self = setmetatable({}, Account)
	self.owner = owner
	self.balance = balance or 0
	self.history = {}
	return self
end

function Account:deposit(amount)
	if amount <= 0 then
		error("deposit must be positive")
	end
	self.balance += amount
	table.insert(self.history, { kind = "deposit", amount = amount })
end

function Account:withdraw(amount)
	if amount > self.balance then
		return false, "insufficient funds"
	end
	self.balance -= amount
	table.insert(self.history, { kind = "withdraw", amount = amount })
	return true
end

function Account:statement()
	local lines = {}
	for index, entry in ipairs(self.history) do
		lines[index] = ("%s %d"):format(entry.kind, entry.amount)
	end
	return table.concat(lines, "\n")
end

local savings = Account.new("sam", 100)
savings:deposit(50)
local ok, reason = savings:withdraw(500)
if not ok then
	print(reason)
end
print(savings:statement())
