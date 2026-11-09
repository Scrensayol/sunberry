local v_u_1 = {}
v_u_1.__index = v_u_1
function v_u_1.new(p2, p3) -- name: new
	-- upvalues: (copy) v_u_1
	local v4 = v_u_1
	local self = setmetatable({}, v4)
	self.owner = p2
	self.balance = p3 or 0
	self.history = {}
	return self
end
function v_u_1:deposit(p5) -- name: deposit
	if p5 <= 0 then
		error("deposit must be positive")
	end
	self.balance += p5
	local history = self.history
	table.insert(history, {
		kind = "deposit",
		amount = p5
	})
end
function v_u_1:withdraw(p6) -- name: withdraw
	if self.balance < p6 then
		return false, "insufficient funds"
	end
	self.balance -= p6
	local history_2 = self.history
	table.insert(history_2, {
		kind = "withdraw",
		amount = p6
	})
	return true
end
function v_u_1:statement() -- name: statement
	local v7 = {}
	for v8, v9 in ipairs(self.history) do
		v7[v8] = ("%s %d"):format(v9.kind, v9.amount)
	end
	return table.concat(v7, "\n")
end
local v10 = v_u_1.new("sam", 100)
v10:deposit(50)
local v11, v12 = v10:withdraw(500)
if not v11 then
	print(v12)
end
print(v10:statement())