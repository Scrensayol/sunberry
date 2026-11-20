print(string.rep == string.rep, string["rep"] == string["rep"], string.rep == string["rep"])
print(table.concat({"a", "b"}, if table["unpack"] then "+" else "-"))
local up = string["upper"]
print(up("x"), utf8["char"](72, 105))
