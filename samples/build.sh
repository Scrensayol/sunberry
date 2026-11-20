#!/bin/sh
# Compiles every sample at -O2 with debug level 1 (no local names, like a published game) and
# decompiles it with sunberry, writing <name>.luauc and <name>.lua next to each source.
# Needs the harness (for its Luau compiler) and sunberry built:
#   cargo build --profile harness -p harness && cargo build --release -p sunberry
set -e
root=$(cd "$(dirname "$0")/.." && pwd)
for source in "$root"/samples/luau/*.luau "$root"/samples/roblox/*.luau; do
	base=${source%.luau}
	# Doors.client.luau is the script Doors
	name=$(basename "$base" | sed 's/\.\(server\|client\)$//')
	"$root/target/harness/sunberry-harness" compile "$source" --profile O2g1 -o "$base.luauc"
	"$root/target/release/sunberry" decompile -e 1 -n "$name" -i "$base.luauc" -o "$base.lua"
done
