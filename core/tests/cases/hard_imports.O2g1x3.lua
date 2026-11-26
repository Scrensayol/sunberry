print(string.rep == string.rep, string["rep"] == string["rep"], string.rep == string["rep"])
print(table.concat({"a", "b"}, if table["unpack"] then "+" else "-"))
local upper = string["upper"]
print(upper("x"), utf8["char"](72, 105))
