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
function v_u_1.deposit(p6, p7) -- name: deposit
	if p7 <= 0 then
		error("deposit must be positive")
	end
	p6.balance = p6.balance + p7
	local history = p6.history
	table.insert(history, {
		["kind"] = "deposit",
		["amount"] = p7
	})
end
function v_u_1.withdraw(p8, p9) -- name: withdraw
	if p8.balance < p9 then
		return false, "insufficient funds"
	end
	p8.balance = p8.balance - p9
	local history_2 = p8.history
	table.insert(history_2, {
		["kind"] = "withdraw",
		["amount"] = p9
	})
	return true
end
function v_u_1.statement(p10) -- name: statement
	local v11 = {}
	for v12, v13 in ipairs(p10.history) do
		v11[v12] = ("%s %d"):format(v13.kind, v13.amount)
	end
	return table.concat(v11, "\n")
end
local v14 = v_u_1.new("sam", 100)
v14:deposit(50)
local v15, v16 = v14:withdraw(500)
if not v15 then
	print(v16)
end
print(v14:statement())