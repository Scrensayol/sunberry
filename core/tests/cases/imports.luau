-- `a.b.c` on a global compiles to an import, looked up once when the script loads; with
-- brackets it's read every time. The two have to stay apart
print(string.rep == string.rep, string["rep"] == string["rep"], string.rep == string["rep"])
print(table.concat({ "a", "b" }, table["unpack"] and "+" or "-"))
local up = string["upper"]
print(up("x"), utf8["char"](72, 105))
