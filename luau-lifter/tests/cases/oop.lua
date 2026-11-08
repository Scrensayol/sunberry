local v_u_1 = {}
v_u_1.__index = v_u_1
function v_u_1.new(p2, p3) -- name: new
	-- upvalues: (copy) v_u_1
	local v4 = v_u_1
	local v5 = setmetatable({}, v4)
	v5.owner = p2
	v5.balance = p3 or 0
	v5.history = {}
	return v5
end
function v_u_1:deposit(p6) -- name: deposit
	if p6 <= 0 then
		error("deposit must be positive")
	end
	self.balance = self.balance + p6
	local history = self.history
	table.insert(history, {
		["kind"] = "deposit",
		["amount"] = p6
	})
end
function v_u_1:withdraw(p7) -- name: withdraw
	if self.balance < p7 then
		return false, "insufficient funds"
	end
	self.balance = self.balance - p7
	local history_2 = self.history
	table.insert(history_2, {
		["kind"] = "withdraw",
		["amount"] = p7
	})
	return true
end
function v_u_1:statement() -- name: statement
	local v8 = {}
	for v9, v10 in ipairs(self.history) do
		v8[v9] = ("%s %d"):format(v10.kind, v10.amount)
	end
	return table.concat(v8, "\n")
end
local v11 = v_u_1.new("sam", 100)
v11:deposit(50)
local v12, v13 = v11:withdraw(500)
if not v12 then
	print(v13)
end
print(v11:statement())