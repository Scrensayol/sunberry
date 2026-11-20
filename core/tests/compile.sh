#!/bin/sh
# Recompiles the snapshot cases' bytecode with the harness's Luau. Sources are the .luau files
# here and in luau-lifter/tests/cases; each is compiled at O1g1 and O2g2.
# Build the harness first: cargo build --profile harness -p harness
set -e
root=$(cd "$(dirname "$0")/../.." && pwd)
for source in "$root"/core/tests/cases/*.luau "$root"/luau-lifter/tests/cases/*.luau; do
	name=$(basename "$source" .luau)
	for profile in O1g1 O2g2; do
		"$root/target/harness/sunberry-harness" compile "$source" --profile "$profile" -o "$root/core/tests/cases/$name.$profile.luauc"
	done
done
