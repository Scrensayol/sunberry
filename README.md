# Sunberry

A fast Luau decompiler that runs happily on a Raspberry Pi. Forked from [shiny](https://github.com/ActualMasterOogway/shiny), which is forked from [medal](https://github.com/shrimp-nz/medal).

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

The workspace needs nightly Rust (see `rust-toolchain`) and a C compiler for mimalloc.

```bash
cargo build --release -p sunberry
```

### Raspberry Pi

Release builds include `sunberry-aarch64-linux-musl`, a static binary that runs on any 64-bit Raspberry Pi OS. To build one tuned for a Pi 5 yourself, cross-compile from a desktop:

```bash
rustup target add aarch64-unknown-linux-musl
# mimalloc needs a C compiler for aarch64 musl, e.g. a wrapper script that runs
# `zig cc -target aarch64-linux-musl "$@"` (drop the --target=... flag cargo passes)
export CC_aarch64_unknown_linux_musl=/path/to/that/wrapper
RUSTFLAGS="-C target-cpu=cortex-a76" \
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
cargo build --release -p sunberry --target aarch64-unknown-linux-musl
```

Drop `target-cpu=cortex-a76` for a binary that also runs on a Pi 4.

## Script

When using `sunberry serve`, you can use the decompiler directly inside of your executor.
These scripts assume you are self-hosting the decompiler.
If you are using some sort of API, make sure to replace `http://localhost:3000` or `http://10.0.2.2:3000` with the correct API base.

### Windows

```lua
getgenv().decompile = function(script_instance)
  local bytecode = getscriptbytecode(script_instance)
  local encoded = crypt.base64encode(bytecode)
  return request(
    {
      Url = "http://localhost:3000/luau/decompile",
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
  return request(
    {
      Url = "http://10.0.2.2:3000/luau/decompile",
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
