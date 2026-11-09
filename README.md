# Sunberry

A fast Luau decompiler built for Linux on ARM64: Raspberry Pis, ARM single-board computers and ARM servers. It ships as one static binary with nothing to install, and `sunberry serve` turns a Pi on your network into a decompile server your executor can call. Forked from [shiny](https://github.com/ActualMasterOogway/shiny), which is forked from [medal](https://github.com/shrimp-nz/medal).

## Downloads

Every release has two Linux ARM64 builds. Both are statically linked, so they run on any distribution as long as the OS is 64-bit (on a Raspberry Pi, that means the 64-bit Raspberry Pi OS).

| File | Runs on |
| --- | --- |
| `sunberry-linux-arm64-universal` | Any 64-bit ARM Linux machine: Raspberry Pi 3, 4, 5 and Zero 2 W, other boards, ARM servers |
| `sunberry-linux-arm64-rpi5` | Raspberry Pi 5 only. It's compiled for the Pi 5's Cortex-A76 cores and stops with "Illegal instruction" on anything older |

Both decompile to exactly the same output. Use the Pi 5 build on a Pi 5 and the universal one everywhere else.

Builds for x86_64 Linux, Windows and macOS are still published, but ARM64 Linux is what Sunberry is built and benchmarked for.

To run it as a service on a Pi, so it starts on boot and restarts if it crashes:

```bash
mkdir -p ~/sunberry ~/.config/systemd/user
install -m 755 sunberry-linux-arm64-rpi5 ~/sunberry/sunberry
cat > ~/.config/systemd/user/sunberry.service << 'EOF'
[Unit]
Description=Sunberry decompile server

[Service]
ExecStart=%h/sunberry/sunberry serve --port 3000
Restart=on-failure

[Install]
WantedBy=default.target
EOF
systemctl --user enable --now sunberry
sudo loginctl enable-linger "$USER"   # keep it running while you're logged out
```

## What's different from shiny

**About 4x faster.** Times for one 1.8 MB Roblox script (1,970 functions, 78,396 lines of output):

| Machine | shiny | Sunberry |
| --- | --- | --- |
| Raspberry Pi 5 (4x Cortex-A76) | 3.35 s | **0.58 s** |
| x86 desktop, 1 thread | 1.87 s | **0.88 s** |
| x86 desktop, 4 threads | 1.87 s | **0.43 s** |

Where it comes from:

- Functions decompile in parallel. Lifting runs in waves and top-level statements are formatted in parallel too.
- mimalloc replaces the system allocator. Decompiling allocates constantly, and musl's malloc made static builds about 60% slower.
- Dominator trees are computed over flat arrays instead of hash maps. The structuring pass recomputes them after every match.
- Two quadratic loops in SSA construction and destruction are now linear.
- Expressions collect the locals they read into one vector per tree instead of nested vectors.

**Readable names.** Locals are named after what they hold when that's clear, instead of `v1` and `v_u_2`:

```lua
local Players = game:GetService("Players")
local CameraInput = require(script.Parent:WaitForChild("CameraInput"))
local part = Instance.new("Part")
local time = tick()
```

Names come from `GetService`, `WaitForChild` and `FindFirst*` arguments, required modules, `Instance.new` class names, string index keys, closure names and globals. They stay unique within a script and never shadow a global the script uses. Bytecode compiled with debug level 2 keeps the original local names, and those are used as-is.

Event handlers and loops get the parameter names you'd expect from Roblox's API:

```lua
Players.PlayerAdded:Connect(function(player)
	player.Chatted:Connect(function(message)
		-- ...
	end)
end)
UserInputService.InputBegan:Connect(function(input, gameProcessed)
	-- ...
end)
for _, descendant in workspace:GetDescendants() do
	-- ...
end
```

**Methods look like methods.** Functions on a class table that use their first parameter as an object print with `:` and `self`, the way they were written:

```lua
function v_u_4:Update(p14)
	local v17 = self:GetHumanoid()
	if self.lastUpdate == nil or p14 > 1 then
```

This never changes behaviour, since `function X:m()` is exactly `X.m = function(self)`.

**Modules are named after their script.** When a module returns a local, that local takes the script's name, so `local v_u_4 = setmetatable({}, BaseCamera)` reads `local ClassicCamera = ...` and `return ClassicCamera`. The CLI uses the input file's name (or `--name`), and `serve` takes a `?name=` parameter.

**Smaller things:** `x = x + 1` prints as `x += 1`, `("a %*"):format(b)` turns back into `` `a {b}` ``, identifier table keys drop the brackets (`{ kind = "x" }`), and constructors name their object `self`.

**Output that compiles, and means the same thing.** All 33,514 scripts in a dump from a Roblox game decompile to code that compiles. Luau has no `goto`, so a function that can't be structured without a `goto` gets its smallest unstructurable region written as a state machine instead. Shiny gets some common loops wrong without any error: a loop that stores what it found and breaks loses the `break`, so it keeps going and ends up with the last match. A local written in a loop and read after it could end up declared inside the loop. Sunberry fixes both.

**Deterministic output.** The same bytecode always decompiles to the same text, at any thread count. In shiny, some statement order depended on memory addresses.

**A better `serve`.** Responses are gzipped when the client asks (executors built on libcurl do by default), so a 2.7 MB result crosses the network as about 370 KB. Bad input returns an error instead of dropping the connection, and long decompiles no longer block other requests.

## Usage

```bash
Usage: sunberry <COMMAND>

Commands:
  decompile  Decompile a single bytecode file, either raw or base64 encoded
  serve      Start a simple web server to handle decompiling
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```bash
sunberry decompile -i Script.luauc -o Script.lua   # Roblox bytecode, encode key 203 by default
sunberry decompile -e 1 -i vanilla.luauc -o out.lua # bytecode from luau-compile
sunberry serve --port 3000                           # listens on all interfaces
```

`RAYON_NUM_THREADS` caps how many cores a decompile uses.

## Building

The workspace needs nightly Rust (see `rust-toolchain`, which rustup picks up by itself) and a C compiler for mimalloc.

### On the ARM64 machine itself

```bash
cargo build --release -p sunberry                                   # runs on this machine
RUSTFLAGS="-C target-cpu=native" cargo build --release -p sunberry  # tuned for this CPU
```

This links against the system's glibc. For a static binary like the release ones, cross-compile instead.

### Cross-compiling from x86_64

```bash
rustup target add aarch64-unknown-linux-musl
# mimalloc needs a C compiler for aarch64 musl, e.g. a wrapper script that runs
# `zig cc -target aarch64-linux-musl "$@"` (drop the --target=... flag cargo passes)
export CC_aarch64_unknown_linux_musl=/path/to/that/wrapper
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld

# universal
cargo build --release -p sunberry --target aarch64-unknown-linux-musl
# Raspberry Pi 5
RUSTFLAGS="-C target-cpu=cortex-a76" \
cargo build --release -p sunberry --target aarch64-unknown-linux-musl
```

`qemu-aarch64-static` runs either one on the build machine, which is a quick check before copying it over. Add `-cpu cortex-a53` to check that the universal build runs on the oldest ARMv8 cores.

### Testing

```bash
cargo test --release -p luau-lifter          # snapshot tests in luau-lifter/tests/cases
UPDATE_SNAPSHOTS=1 cargo test --release -p luau-lifter   # accept intended output changes
```

To check a change against real scripts, dump some bytecode to a folder and compare the old and new builds over it. The script reports changed outputs, anything that decompiles differently across thread counts and, when `luau-compile` is on your PATH, output that stopped compiling:

```bash
scripts/regress.sh old/sunberry target/release/sunberry path/to/bytecode
```

## Script

When using `sunberry serve`, you can use the decompiler directly inside of your executor.
These scripts assume you are self-hosting the decompiler.
When it runs on a Pi or another machine on your network, replace `http://localhost:3000` with that machine's address, such as `http://192.168.1.50:3000`. `serve` listens on all interfaces, so nothing else needs setting up.
If you are using some sort of API, make sure to replace `http://localhost:3000` or `http://10.0.2.2:3000` with the correct API base.

### Windows

```lua
getgenv().decompile = function(script_instance)
  local bytecode = getscriptbytecode(script_instance)
  local encoded = crypt.base64encode(bytecode)
  local name = game:GetService("HttpService"):UrlEncode(script_instance.Name)
  return request(
    {
      Url = "http://localhost:3000/luau/decompile?name=" .. name,
      Method = "POST",
      Body = encoded
    }
  ).Body
end

loadstring(game:HttpGet("https://raw.githubusercontent.com/luau/SynSaveInstance/main/saveinstance.luau"))()({
  mode = "scripts",
  NilInstances = true,
})
```

### Android Emulator

> [!NOTE]
>
> - If you have a proxy enabled in WiFi settings, add `10.0.2.2` to the exceptions or disable the proxy
> - If you are using a VPN, allow LAN connections

Run the web-server on your actual machine.

If you are using an emulator, `localhost` will not work since that will refer to the emulator's device.
Instead, we must use `10.0.2.2` instead.
To test if everything is working, on your actual machine [http://localhost:3000/] should load fine, and on the emulator's browser, [http://10.0.2.2:3000/] should load. If your emulator is giving you HTTP connection fail errors, then you can try [ngrok](https://ngrok.com/) instead.

```lua
getgenv().decompile = function(script_instance)
  local bytecode = getscriptbytecode(script_instance)
  local encoded = crypt.base64encode(bytecode)
  local name = game:GetService("HttpService"):UrlEncode(script_instance.Name)
  return request(
    {
      Url = "http://10.0.2.2:3000/luau/decompile?name=" .. name,
      Method = "POST",
      Body = encoded
    }
  ).Body
end

loadstring(game:HttpGet("https://raw.githubusercontent.com/luau/SynSaveInstance/main/saveinstance.luau"))()({
  mode = "scripts",
  NilInstances = true,
})
```

## License

The [original license](https://github.com/shrimp-nz/medal/blob/main/LICENSE.txt) for medal is retained, but the license for this fork has been changed to stop people from profiting from open source work for free.

## Credits

All credits to the medal project goes to in honor and memory of

- Jujhar Singh (KowalskiFX)
- Mathias Pedersen (Customality)

Keep the Singh and Pedersen family in your guys' prayers, we love you both.
