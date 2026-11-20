# Samples

Six scripts with their bytecode and what sunberry makes of it. Each `<name>.luau` is the source, `<name>.luauc` is that source compiled at `-O2` with debug level 1 (no local names, like a published game), and `<name>.lua` is sunberry's output.

| Script | What it exercises |
| --- | --- |
| [`luau/inventory`](luau/inventory.luau) | a class with methods, `continue`, compound assignment, interpolated strings, a sort callback |
| [`luau/pathfinding`](luau/pathfinding.luau) | A* with a binary heap: nested loops, `break`, multiple assignment, a local function inlined at `-O2` |
| [`luau/calculator`](luau/calculator.luau) | a tokenizer and recursive-descent parser: string patterns, recursion, `pcall` and `error` |
| [`roblox/Leaderboard.server`](roblox/Leaderboard.server.luau) | DataStore saving with retries, `leaderstats`, `PlayerAdded`, `BindToClose` |
| [`roblox/Doors.client`](roblox/Doors.client.luau) | `CollectionService` tags, `ProximityPrompt`, `TweenService`, a debounce table |
| [`roblox/CameraShake`](roblox/CameraShake.luau) | a ModuleScript class driven by `RenderStepped`, Perlin noise |

The plain Luau samples run as they are (`luau luau/pathfinding.luau`), and the round-trip harness checks them on every CI run: each is compiled at six optimization and debug levels, decompiled, recompiled, and run, and the output has to print exactly what the source printed. The Roblox samples need the engine to run, so for those the check is that the output compiles.

Here's what `CameraShake.luau` comes back as:

```lua
local RunService = game:GetService("RunService")
local CameraShake = {}
CameraShake.__index = CameraShake
function CameraShake.new(p1)
	local self = setmetatable({}, CameraShake)
	self.camera = p1 or workspace.CurrentCamera
	self.shakes = {}
	self.seed = math.random() * 1000
	self.connection = nil
	return self
end
function CameraShake:Shake(p2, p3, p4)
	table.insert(self.shakes, {magnitude = p2, duration = p3, rotation = p4 or p2 * 0.5, started = os.clock()})
	if not self.connection then
		self.connection = RunService.RenderStepped:Connect(function()
			self:_step()
		end)
	end
end
```

Parameter names aren't in the bytecode at debug level 1, so they come back as `p1`, `p2`. Everything else here is recovered: the module's name from the script, `self` from the constructor, methods from how they're called, and the service from `GetService`.

To rebuild after a decompiler change:

```bash
cargo build --profile harness -p harness && cargo build --release -p sunberry
samples/build.sh
```
