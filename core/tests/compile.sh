#!/bin/sh
# Recompiles the snapshot cases' bytecode with Rosetta's Luau
# (https://github.com/Scrensayol/rosetta). Sources are the .luau files here and in
# luau-lifter/tests/cases; each is compiled at O1g1 and O2g2.
# Build Rosetta first (cargo build --release in its checkout); ROSETTA points at the binary,
# by default ../rosetta next to this repo.
set -e
root=$(cd "$(dirname "$0")/../.." && pwd)
rosetta=${ROSETTA:-$root/../rosetta/target/release/rosetta}
for source in "$root"/core/tests/cases/*.luau "$root"/luau-lifter/tests/cases/*.luau; do
	name=$(basename "$source" .luau)
	for profile in O1g1 O2g2; do
		"$rosetta" compile "$source" --profile "$profile" -o "$root/core/tests/cases/$name.$profile.luauc"
	done
done
