# Ice and Fire — validation ladder (slice 1, SDK package)

Build **0b14f8acfd97fa2c750e42795249dcc981178ca4** (game 26.3), mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2. Every build/test ran inside a
`ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`; no `--target-dir` was passed.

| # | Step | Command | Evidence | Result |
|---|---|---|---|---|
| 1 | runtime carried byte-identical | `cmp src/runtime.rs evidence/…/pre-sdk/mixins/iceandfire/runtime.rs` | `evidence/iceandfire/runtime-identity.log` | **exit 0** (identical), sha256 `b28ea3dc…02279` |
| 2 | runtime + glue tests | `cargo test --offline` | `evidence/iceandfire/runtime-test.log` | **18 passed**, 0 failed (15 runtime + 3 glue) |
| 3 | lints | `cargo clippy --offline --all-targets -- -D warnings` | `evidence/iceandfire/clippy.log` | **exit 0**, 0 warnings |
| 4 | format | `rustfmt --edition 2024 --check src/*.rs` | run output | **exit 0** (the fetched SDK is not formatted: `cargo fmt --all` would rewrite `.horizon/sdk/`) |
| 5 | package build | `cargo horizon-mod build` | `evidence/iceandfire/hmod-build.log` | `Built ./target/horizon-mod/iceandfire.hmod`, **exit 0** |
| 6 | package check | `cargo horizon-mod check iceandfire.hmod` | `evidence/iceandfire/hmod-check.log` | **exit 0**: `native-mod, format native-1/0b14f8a…/e59da90…, for 26.3`, 41,200 B |
| 7 | fits the build | `cargo horizon-mod fits iceandfire.hmod` | `evidence/iceandfire/hmod-fits.log` | **exit 0**: `0b14f8a  made for` |
| 8 | entries | `unzip -l iceandfire.hmod` | `evidence/iceandfire/hmod-entries.log` | **21 entries**: manifest, Cargo, 3 sources, 2 licence files, README, `uses.json`, 6 assets, 5 data |
| 9 | determinism | two `cargo horizon-mod build` runs | `evidence/iceandfire/hmod-determinism.log` | same sha256 `6832d305…7f63` both times |
| 10 | API probe (damage / mount / goals) | `cargo check` in `evidence/iceandfire/api-probe/` | `evidence/iceandfire/api-probe.log` | **exit 0**: `EntityRef::hurt`, `EntityRef::start_riding` and a custom `Goal` type-check on a hook's `ServerHost` |
| 11 | skills gate | `python3 ports/tools/check-skills.py --notes ports/iceandfire` | run output | `SKILLS GATE: PASS` |

## Deliverables

```
ports/iceandfire/iceandfire.hmod   41,200 B
   b4023b521155e131ae7f17005c79fab8c5735a0ccaf86e2702692fdb59712109   21 entries
```

The package holds `horizon-mod.toml`, `Cargo.toml`, `Cargo.lock`, `src/{lib,glue,runtime}.rs`,
`LICENSE`, `LICENSE-NOTICE.md`, `README.md`, the generated `uses.json`, and the carried
`assets/iceandfire/…` and `data/iceandfire/…`. It is the whole mod: content and code.

## Not covered by this ladder

- **No in-game leg.** The SDK format runs through `horizon-launcher` (`cargo horizon-mod run`
  answers *"code mods run through the launcher's native build: horizon-launcher native-build .
  --trust-native-code"*), which is not installed on this machine, and the launcher is a
  separate product (`playhorizon.dev`), not part of the release assets. So no `horizon-ctl`
  transcript was produced for this migration, and none is claimed.
- **The last in-game evidence is the `.hnmod` slice** (old pin `1d63e0a`, `tools/e2e.sh`,
  `PASS` + `EXIT:0`, logs at `evidence/iceandfire/e2e-{dev,embedded,selftest}.log`, source at `evidence/iceandfire/pre-sdk/`): with the egg on stone it aged
  **0** in the same window that took **3** ages on magma; the hatch produced `Tamed 1b`, an
  `Owner` and health **20.0** (=500×0.04); the dragon rose **68.3 → 71.0** in 30 ticks while
  a cow and a wandering trader kept their height; a wild dragon closed **16.15 → 1.35** and
  held; the selftest leg failed with `EXIT:1`; the embedded-content leg loaded the content
  from the bundle. Those assertions are the behaviour this migration must preserve, and the
  glue that produces them is the same five moments (place → arm → scheduled tick → hatch →
  entity tick), now expressed against the SDK.
- **A world reload with a live tamed dragon** (does the `iceandfire:*` tag set survive a
  save/load?) — the `.hnmod` e2e did not reload either; the tags ride the entity's saved
  data through `summon`/`nbt`.
