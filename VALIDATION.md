# Ice and Fire — validation ladder (slice 2: the bite, and the first in-game leg)

Build **`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (game 26.3, Nightly 2026-11-20), mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, mod API 0.2, launcher 0.2.92, rust 1.99.1. Every
cargo/test/build step ran inside a `ports/tools/build-slot.sh` lease with `CARGO_INCREMENTAL=0`;
no `--target-dir` was passed. (`cargo horizon-mod` exists only on this box, so those steps run
with `BUILD_LOCAL=1`; the unit tests/clippy there too.)

## The slice

The dragon's **bite** — `notes.md` §"Slice 2" has the rule, the upstream lines and the red→green
record. Package version **0.2.1**, repinned from `0b14f8a` to `0a875da` (`none of the 176 items
it uses changed`).

## Ladder

| # | Step | Command | Evidence | Result |
|---|---|---|---|---|
| 1 | red: the bite's rule | `cargo test --offline` before the code | `evidence/iceandfire/sdk-test-red.log` | **exit 101**: `error[E0425]: cannot find value `BITE_ANIM_TICKS` in this scope` … `23 previous errors` |
| 2 | tests | `cargo test --offline` | `evidence/iceandfire/sdk-test.log` | **26 passed**, 0 failed (18 carried + 8 bite) |
| 3 | lints | `cargo clippy --offline --all-targets -- -D warnings` | `evidence/iceandfire/sdk-clippy.log` | **exit 0**, 0 warnings |
| 4 | format | `rustfmt --edition 2024 --check src/*.rs` | run output | **exit 0** (the fetched SDK is never formatted) |
| 5 | build ×2 | `rm -f Cargo.lock && cargo horizon-mod build` | `sdk-build-1.log`, `sdk-build-2.log` | both **exit 0**, sha256 `91b36a16…2d1d769` both times, **`cmp` identical** |
| 6 | package check | `cargo horizon-mod check iceandfire.hmod` | `evidence/iceandfire/sdk-check.log` (prints the file's sha first) | **exit 0**: `iceandfire 0.2.1 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, `43,248 bytes, SHA-256 91b36a16…` |
| 7 | fits, the pin | `cargo horizon-mod fits iceandfire.hmod --build 0a875da` | `evidence/iceandfire/sdk-fits.log` (prints the sha) | **exit 0**: `0a875da  made for` |
| 8 | fits, a different build | `cargo horizon-mod fits iceandfire.hmod --build 0b14f8a` | `evidence/iceandfire/sdk-fits-other.log` (prints the sha) | **exit 0**: `0b14f8a  fits` |
| 9 | gate | `python3 ports/tools/verify-sdk-package.py iceandfire` | run output | **1/1** (package, runtime byte-identical, no packed `Cargo.lock`, 26 tests, 0 clippy warnings, check, fits, skills) |
| 10 | skills gate | `python3 ports/tools/check-skills.py --notes ports/iceandfire --strict` | run output | **PASS** |

## In-game leg (`tools/e2e.sh`, launcher-built game)

`ports/tools/launcher-game.sh build iceandfire` → `native-0a875da-…`; the exports come from
`launcher-game.sh env iceandfire`; each leg runs in a slot lease (`--step e2e`). The End void at
x=200, a stone floor at y=100, headless, creative, the world seed 20261105.

| Leg | Log | Verdict |
|---|---|---|
| default (A–F) | `evidence/iceandfire-e2e/e2e-default.log` | **`PASS` + `EXIT:0`** on the delivered package (sha `91b36a16…` printed in the log) |
| red, the previous package (`0.2.0`, no bite) | `evidence/iceandfire-e2e/e2e-red-bite-absent.log` | **`FAIL` + `EXIT:1`**: `the wild dragon never hurt the player over 90 ticks (health 20.0 … 20.0)` |
| selftest (`SELFTEST=1`) | `evidence/iceandfire-e2e/e2e-selftest.log` | **`FAIL` + `EXIT:1`**: the tamed control's inversion trips |
| control game (`CONTROL_ONLY=1`, `GAME=$CONTROL_GAME`) | `evidence/iceandfire-e2e/e2e-control.log` | **`FAIL` + `EXIT:1`**: the ids are unknown without the package |
| log gate | `python3 ports/tools/check-e2e-logs.py iceandfire-e2e --strict` | **4/4 ok**, exit 0 |

What the default leg asserts (each parses a `ctl` reply; see the script's header):

* **A the bite** — survival player at 20.0; a wild `iceandfire:fire_dragon` six blocks away;
  the health series (every 5 ticks for 90) drops **exactly 1.0** per step: `20.0->19.0
  19.0->18.0 18.0->17.0 17.0->16.0`.
* **B the tamed control** — a dragon with `{"iceandfire:Tamed":1b}` three blocks away leaves
  the health at 16.0 for the same window; a `/kill`ed dragon is gone afterwards.
* **C the carried content** — 3 egg items given and read back; `iceandfire:no_such_item`
  refused; the item places `iceandfire:dragon_egg[age=0]`.
* **D the flame differential** — after the same window: egg on stone `age=0`, egg on magma `age=7`.
* **E the hatch** — the magma egg's 7,200 ticks (`age 29/49/69` at 2,000/4,000/6,000 ticks) end
  in air and one dragon with `20.0` health, `"iceandfire:Tamed": 1b` and
  `"iceandfire:Owner": [I; …]`.
* **F the loot table** — a vanilla barrel broken beside it drops 1 (`{"minecraft:barrel":1}`),
  the egg block drops 1 (`{"iceandfire:dragonegg_red":1}`), and `tools/data_shape.py` passes on
  the package.

Deliverable:

```
ports/iceandfire/iceandfire.hmod   43,248 B
   91b36a165ade990bb7b4b7b52554b6c768eceeef435e8248039ae53e32d1d769   20 entries
```

`horizon-mod.toml`, `Cargo.toml`, `src/{lib,glue,runtime}.rs`, `LICENSE`, `LICENSE-NOTICE.md`,
`README.md`, the generated `uses.json`, and the carried `assets/iceandfire/…` +
`data/iceandfire/…`. No `Cargo.lock` is packed.

## Not covered by this ladder

* **The breath weapon and the rider controls** (gap 7): the cone of fire blocks and the fire
  charge projectile, and any mount control surface.
* **Ageing/growth, the dragons' own loot tables, the other 21 entity types, the roosts and
  their worldgen, the other egg colours, the gear families, the bestiary** (gap 8).
* **The dragon's own model and texture** (gap 3) — it draws its template's (a wandering
  trader's) model. `--headless` legs cannot judge a look; the block's and item's models are
  carried but not asserted.
* **A world reload with a live tamed dragon** — the tags ride the entity's saved data through
  `summon`/`nbt`, which the hatch leg reads back; a save/quit/relaunch of that dragon is still
  untested.
* **A visual leg** — everything here is `--headless`.

## Pre-SDK record (the `.hnmod` slice at `1d63e0a`, kept for reading)

`evidence/iceandfire/e2e-{dev,embedded,selftest}.log`, source at `evidence/iceandfire/pre-sdk/`:
with the egg on stone it aged **0** in the same window that took **3** ages on magma; the hatch
produced `Tamed 1b`, an `Owner` and health **20.0**; the dragon rose **68.3 → 71.0** in 30 ticks
while a cow and a wandering trader kept their height; a wild dragon closed **16.15 → 1.35** and
held; the selftest leg failed with `EXIT:1`.
