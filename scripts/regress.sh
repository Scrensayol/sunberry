#!/usr/bin/env bash
# Compares two sunberry builds over the *.luauc bytecode files in a folder.
#
#   scripts/regress.sh <old binary> <new binary> <bytecode dir> [encode key]
#
# For every file it reports whether the new build's output differs from the old one's,
# whether the new output decompiles identically with 8 threads twice and with 1 thread,
# and, if a Luau compiler is available, whether the output of each build still compiles.
# The syntax check runs `$LUAU_CHECK <file>` (default: `luau-compile --binary`) and counts
# a nonzero exit as a syntax error. The encode key defaults to 203, Roblox's.
set -euo pipefail

if [ $# -lt 3 ]; then
	sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//'
	exit 1
fi

old=$(realpath "$1")
new=$(realpath "$2")
dir=$3
key=${4:-203}
check=${LUAU_CHECK:-luau-compile --binary}
if ! command -v "${check%% *}" > /dev/null; then
	echo "no ${check%% *} on PATH, skipping the syntax check (set LUAU_CHECK to use another compiler)"
	check=""
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
export old new key check work

one() {
	local file=$1 name out_old out_new
	name=$(basename "$file")
	out_old="$work/$name.old.lua"
	out_new="$work/$name.new.lua"
	if ! timeout 120 "$old" decompile -e "$key" -i "$file" -o "$out_old" > /dev/null 2>&1 \
		|| ! RAYON_NUM_THREADS=8 timeout 120 "$new" decompile -e "$key" -i "$file" -o "$out_new" > /dev/null 2>&1; then
		echo "decompile-failed $name"
		return
	fi

	local again single
	again=$(RAYON_NUM_THREADS=8 timeout 120 "$new" decompile -e "$key" -i "$file" -o /dev/stdout 2> /dev/null | md5sum)
	single=$(RAYON_NUM_THREADS=1 timeout 120 "$new" decompile -e "$key" -i "$file" -o /dev/stdout 2> /dev/null | md5sum)
	if [ "$(md5sum < "$out_new")" != "$again" ] || [ "$again" != "$single" ]; then
		echo "nondeterministic $name"
	fi

	if cmp -s "$out_old" "$out_new"; then echo "same $name"; else echo "changed $name"; fi

	if [ -n "$check" ]; then
		local old_ok=1 new_ok=1
		$check "$out_old" > /dev/null 2>&1 || old_ok=0
		$check "$out_new" > /dev/null 2>&1 || new_ok=0
		if [ $old_ok = 1 ] && [ $new_ok = 0 ]; then echo "newly-broken $name"; fi
		if [ $old_ok = 0 ] && [ $new_ok = 1 ]; then echo "newly-fixed $name"; fi
		if [ $new_ok = 0 ]; then echo "invalid $name"; fi
	fi
	rm -f "$out_old" "$out_new"
}
export -f one

find "$dir" -type f -name "*.luauc" -print0 | xargs -0 -P "$(nproc)" -I{} bash -c 'one "$1"' _ {} > "$work/results"

count() { grep -c "^$1 " "$work/results" || true; }
echo "scripts:          $(find "$dir" -type f -name "*.luauc" | wc -l)"
echo "same output:      $(count same)"
echo "changed output:   $(count changed)"
echo "decompile failed: $(count decompile-failed)"
echo "nondeterministic: $(count nondeterministic)"
if [ -n "$check" ]; then
	echo "invalid output:   $(count invalid)"
	echo "newly broken:     $(count newly-broken)"
	echo "newly fixed:      $(count newly-fixed)"
fi
for kind in nondeterministic newly-broken decompile-failed; do
	if grep -q "^$kind " "$work/results"; then
		echo
		echo "$kind:"
		grep "^$kind " "$work/results" | cut -d' ' -f2- | sort | head -20
	fi
done
if grep -q "^nondeterministic \|^newly-broken " "$work/results"; then exit 1; fi
