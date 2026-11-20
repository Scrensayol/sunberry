-- A tokenizer and recursive-descent parser for arithmetic, with variables and errors.

local function tokenize(source)
	local tokens = {}
	local i = 1
	while i <= #source do
		local c = source:sub(i, i)
		if c:match("%s") then
			i += 1
		elseif c:match("%d") then
			local number = source:match("^%d+%.?%d*", i)
			table.insert(tokens, { kind = "number", value = tonumber(number) })
			i += #number
		elseif c:match("[%a_]") then
			local name = source:match("^[%a_][%w_]*", i)
			table.insert(tokens, { kind = "name", value = name })
			i += #name
		elseif c:match("[%+%-%*/%^%(%)=]") then
			table.insert(tokens, { kind = c })
			i += 1
		else
			error(`unexpected character '{c}' at {i}`, 0)
		end
	end
	table.insert(tokens, { kind = "end" })
	return tokens
end

local Parser = {}
Parser.__index = Parser

function Parser.new(tokens, variables)
	return setmetatable({ tokens = tokens, pos = 1, variables = variables }, Parser)
end

function Parser:peek()
	return self.tokens[self.pos].kind
end

function Parser:take(kind)
	local token = self.tokens[self.pos]
	if kind and token.kind ~= kind then
		error(`expected {kind}, got {token.kind}`, 0)
	end
	self.pos += 1
	return token
end

local PRECEDENCE = { ["+"] = 1, ["-"] = 1, ["*"] = 2, ["/"] = 2, ["^"] = 3 }

function Parser:primary()
	local token = self:take()
	if token.kind == "number" then
		return token.value
	elseif token.kind == "name" then
		local value = self.variables[token.value]
		if value == nil then
			error(`unknown variable {token.value}`, 0)
		end
		return value
	elseif token.kind == "-" then
		return -self:expression(3)
	elseif token.kind == "(" then
		local value = self:expression(0)
		self:take(")")
		return value
	end
	error(`unexpected {token.kind}`, 0)
end

function Parser:expression(minimum)
	local left = self:primary()
	while true do
		local op = self:peek()
		local precedence = PRECEDENCE[op]
		if not precedence or precedence <= minimum and op ~= "^" or precedence < minimum then
			return left
		end
		self:take()
		local right = self:expression(op == "^" and precedence - 1 or precedence)
		if op == "+" then
			left += right
		elseif op == "-" then
			left -= right
		elseif op == "*" then
			left *= right
		elseif op == "/" then
			if right == 0 then
				error("division by zero", 0)
			end
			left /= right
		else
			left ^= right
		end
	end
end

local function run(line, variables)
	local tokens = tokenize(line)
	local target
	if tokens[2].kind == "=" then
		target = tokens[1].value
		table.remove(tokens, 1)
		table.remove(tokens, 1)
	end
	local parser = Parser.new(tokens, variables)
	local value = parser:expression(0)
	parser:take("end")
	if target then
		variables[target] = value
	end
	return value
end

local variables = { pi = math.pi }
for _, line in { "x = 2 + 3 * 4", "y = (x - 4) ^ 2 / 2", "x * y - -3", "r = 2", "pi * r ^ 2", "1 / (x - 14)", "z + 1", "3 $ 4" } do
	local ok, result = pcall(run, line, variables)
	print(line, "=>", if ok then string.format("%.4g", result) else `error: {result}`)
end
