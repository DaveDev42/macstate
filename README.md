# macstate

macOS system signals as JSON — a CLI and a Rust library.

Surfaces signals that aren't directly readable from the shell:

- **Network** — Low Data Mode (`constrained`), expensive link (cellular/hotspot), interface kind
- **Power** — AC vs battery, battery percent, Low Power Mode, Energy Mode (automatic/low/high)

`low_power_mode` reports the *currently active* state (via
`NSProcessInfo.isLowPowerModeEnabled`); `energy_mode` reports the
*configured preference* for the current power source (AC or Battery)
read from IOKit's active PM preferences — the same source `pmset` uses.

macOS only.

## Install

From crates.io (requires a Rust toolchain):

```sh
cargo install macstate
```

Prebuilt binary via [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) (no toolchain needed):

```sh
cargo binstall macstate
```

Or pin to the bleeding edge from git:

```sh
cargo install --git https://github.com/DaveDev42/macstate macstate
```

Update any of the above with `--force`.

## Usage

Default: print everything as JSON and exit.

```sh
$ macstate
{
  "network": {
    "constrained": false,
    "expensive": false,
    "interface": "wifi"
  },
  "power": {
    "source": "ac",
    "battery_percent": 87,
    "low_power_mode": false,
    "energy_mode": "automatic"
  }
}
```

Subsets:

```sh
macstate --network
macstate --power
```

Single value at a dotted path:

```sh
$ macstate -q network.constrained
false
$ macstate -q power.battery_percent
87
```

Shell guard via exit code (true → 0, false → 1):

```sh
macstate --check network.constrained && echo "low data mode on"
```

JSON Schema for the output (useful for AI agents and downstream tools):

```sh
macstate --schema           # print to stdout
macstate --schema > schema.json
```

The schema is generated from the Rust types at runtime via
[`schemars`](https://crates.io/crates/schemars), so it always matches
what the CLI actually emits.

## Library

```toml
[dependencies]
macstate-core = "0.0"
```

```rust
let state = macstate_core::State::collect();
println!("{}", state.power.battery_percent.unwrap_or(0));
```

## License

BSD 3-Clause. See [LICENSE](LICENSE).
