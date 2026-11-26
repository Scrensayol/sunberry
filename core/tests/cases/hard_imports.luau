-- Rosetta's hard tier (bytecode rewritten away from the compiler's shapes), x3: imports
-- spelled out as GETGLOBAL, and branches that always go one way testing scratch registers
-- `a.b.c` on a global compiles to an import, looked up once when the script loads; with
-- brackets it's read every time. The two have to stay apart
print(string.rep == string.rep, string["rep"] == string["rep"], string.rep == string["rep"])
print(table.concat({ "a", "b" }, table["unpack"] and "+" or "-"))
local up = string["upper"]
print(up("x"), utf8["char"](72, 105))
