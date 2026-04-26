# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`macstate` exposes macOS system signals (network, power) that aren't directly readable from the shell, as JSON. macOS-only (`#[cfg(target_os = "macos")]`); other targets compile with safe defaults.

## Build / run / release

- Build: `cargo build` (workspace) — also `--release`.
- Run CLI in dev: `./target/debug/macstate [--network|--power|-q PATH|--check PATH|--schema]`.
- Validate output against the embedded schema:
  `uvx --from jsonschema python -c "import json,subprocess,jsonschema; s=json.loads(subprocess.check_output(['./target/release/macstate','--schema'])); d=json.loads(subprocess.check_output(['./target/release/macstate'])); jsonschema.validate(d,s); print('OK')"`
- Release: `cargo release patch --execute --no-confirm` (also `minor`/`major`). This bumps every crate to the same next version (lockstep, `shared-version = true` in `release.toml`), updates `path` deps' `version = "..."` fields, commits `release vX.Y.Z`, tags `vX.Y.Z`, and pushes. The `release.yml` workflow then builds aarch64+x86_64 binaries, creates the GitHub Release with sha256s, and publishes all three crates to crates.io via Trusted Publishing OIDC. **Local `cargo publish` is disabled by config — never run it manually.**

## Workspace layout (3 crates, lockstep versioned)

```
macstate-sys     hand-written FFI: CoreFoundation, IOKit (IOPS*, IOPMCopyActivePMPreferences),
                 libdispatch, Network.framework (NWPathMonitor), and a tiny objc2 bridge for
                 NSProcessInfo. Owning wrappers (CFOwned, DispatchOwned) call the right
                 *Release on drop.
macstate-core    domain library; turns sys calls into serde-serializable State { network, power }.
macstate-cli     `macstate` binary (note the [[bin]] name differs from the crate name).
                 Owns schema.json (embedded via include_str!) and the cargo-binstall metadata.
```

Dep policy: `macstate-core` deliberately keeps a tiny tree (`serde` + `macstate-sys` → `block2` + `objc2`) so it's cheap for downstream consumers (Tauri etc.) to pull in. **Do not add `objc2-foundation` / `objc2-io-kit` / `objc2-core-foundation` / `dispatch2` back** — they were intentionally replaced with hand-written FFI in `macstate-sys`. New macOS APIs go into `macstate-sys` first.

## Signal-by-signal sources of truth

When changing or adding a signal, mirror the existing pattern: raw FFI in `macstate-sys`, typed wrapper + `Serialize` enum in `macstate-core`, JSON Schema entry in `crates/macstate-cli/schema.json`.

- `network.*`: `NWPathMonitor` one-shot snapshot in `nwpath.rs` — start monitor, wait on a `dispatch_semaphore` for the first update handler firing, then cancel. The monitor is push-based; we synchronize with a semaphore + `AtomicBool` (`fired`) so only the first delivery is captured.
- `power.source` / `battery_percent`: `IOPSCopyPowerSourcesInfo` + `IOPSGetProvidingPowerSourceType` + per-source dict walk.
- `power.low_power_mode`: `NSProcessInfo.isLowPowerModeEnabled` via `objc2::msg_send!` — current *active* state.
- `power.energy_mode`: `IOPMCopyActivePMPreferences` (private but stable; same source `pmset(8)` reads). The dict's `LowPowerMode` key is **not a boolean** — it's the unified `pmset powermode` indicator (0=automatic, 1=low, 2=high). The sibling `HighPowerMode` key exists but is unused on current macOS. Returns `EnergyMode::Unknown` rather than silently lying with `Automatic` when the value is missing or unrecognized.

## Schema

`--schema` is generated at runtime from the public types via `schemars` — there is no checked-in schema file. The `schema` feature on `macstate-core` (default off) gates the `JsonSchema` derives so library consumers don't pay the dep. `macstate-cli` enables it.

When adding a field or enum variant: put the human description in a doc comment or `#[cfg_attr(feature = "schema", schemars(description = "..."))]` so it shows up in the generated schema. Don't write a parallel schema file.

## CI / release workflow gotchas

- `release.yml` triggers on `v*` tags only.
- `id-token: write` permission is required for crates.io Trusted Publishing OIDC.
- Trusted Publishers are registered per-crate on crates.io for repo `DaveDev42/macstate`, workflow filename `release.yml`. If you rename the workflow file, re-register.
- `cargo-binstall` metadata in `crates/macstate-cli/Cargo.toml` (`pkg-url`, `bin-dir`) must stay in sync with the workflow's archive layout (`macstate-vX.Y.Z-{target}/macstate`). Verify after any workflow change with `cargo binstall macstate@<ver> --no-confirm --dry-run`.
