-- a table built from a template and then filled in: `next` visits keys in the order they
-- went in, so the constructor has to keep the source's order
local bag = { items = { "x", "y" }, hidden = true }
for k, v in bag do
	print(k, type(v))
end
