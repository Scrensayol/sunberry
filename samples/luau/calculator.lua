local function tokenize(p1)
	local v2 = {}
	local v3 = 1
	while v3 <= #p1 do
		local v4 = p1:sub(v3, v3)
		if v4:match("%s") then
			v3 += 1
		elseif v4:match("%d") then
			local v5 = p1:match("^%d+%.?%d*", v3)
			table.insert(v2, {kind = "number", value = tonumber(v5)})
			v3 += #v5
		elseif v4:match("[%a_]") then
			local v6 = p1:match("^[%a_][%w_]*", v3)
			table.insert(v2, {kind = "name", value = v6})
			v3 += #v6
		elseif v4:match("[%+%-%*/%^%(%)=]") then
			table.insert(v2, {kind = v4})
			v3 += 1
		else
			error(`unexpected character '{v4}' at {v3}`, 0)
		end
	end
	table.insert(v2, {kind = "end"})
	return v2
end
local v7 = {}
v7.__index = v7
function v7.new(p8, p9)
	return (setmetatable({pos = 1, tokens = p8, variables = p9}, v7))
end
function v7:peek()
	return self.tokens[self.pos].kind
end
function v7:take(p10)
	local v11 = self.tokens[self.pos]
	if p10 and v11.kind ~= p10 then
		error(`expected {p10}, got {v11.kind}`, 0)
	end
	self.pos += 1
	return v11
end
function v7:primary()
	local v12 = self:take()
	if v12.kind == "number" then
		return v12.value
	end
	if v12.kind == "name" then
		local v13 = self.variables[v12.value]
		if v13 == nil then
			error(`unknown variable {v12.value}`, 0)
		end
		return v13
	end
	if v12.kind == "-" then
		return -self:expression(3)
	end
	if v12.kind == "(" then
		local v14 = self:expression(0)
		self:take(")")
		return v14
	end
	error(`unexpected {v12.kind}`, 0)
end
local v15 = {
	["+"] = 1,
	["-"] = 1,
	["*"] = 2,
	["/"] = 2,
	["^"] = 3
}
function v7:expression(p16)
	local v17 = self:primary()
	while true do
		local v18 = self:peek()
		local v19 = v15[v18]
		if not (v19 and (not (v19 <= p16) or v18 == "^")) or v19 < p16 then
			return v17
		end
		self:take()
		local v20 = self:expression(v18 == "^" and v19 - 1 or v19)
		if v18 == "+" then
			v17 += v20
		elseif v18 == "-" then
			v17 -= v20
		elseif v18 == "*" then
			v17 *= v20
		elseif v18 == "/" then
			if v20 == 0 then
				error("division by zero", 0)
			end
			v17 /= v20
		else
			v17 ^= v20
		end
	end
end
local function run(p21, p22)
	local v23 = tokenize(p21)
	local value
	if v23[2].kind == "=" then
		value = v23[1].value
		table.remove(v23, 1)
		table.remove(v23, 1)
	end
	local v24 = v7.new(v23, p22)
	local v25 = v24:expression(0)
	v24:take("end")
	if value then
		p22[value] = v25
	end
	return v25
end
local v26 = {pi = 3.141592653589793}
for _, v in {
	"x = 2 + 3 * 4",
	"y = (x - 4) ^ 2 / 2",
	"x * y - -3",
	"r = 2",
	"pi * r ^ 2",
	"1 / (x - 14)",
	"z + 1",
	"3 $ 4"
} do
	local success, result = pcall(run, v, v26)
	print(v, "=>", if success then string.format("%.4g", result) else `error: {result}`)
end
