# Ice and Fire Community Edition for Horizon — the fire dragon and its bite

One dragon, end to end: the **red fire dragon's egg** (the jar's own spawn mechanism),
its flame-gated incubation, the **tamed, owned fire dragon** it hatches, that dragon's
**flight and pursuit**, and the **bite** it lands on a player it hunts.

Ported from **Ice And Fire Community Edition** `2.0-beta.17` (`iceandfire`) by
**IAFEnvoy, xiaowu, uoay** — <https://github.com/IAFEnvoy/IceAndFire-CE>.
Licence **LGPL-3.0**; this port is a derivative under the same licence and its
complete source is published with it. See `LICENSE` and `LICENSE-NOTICE.md`.

This is the **SDK-format package** (mod API 0.2, Horizon build `0a875da`, Nightly
2026-11-20): one flat `.hmod` holding the manifest, the Rust crate and the carried
content. The `.hnmod` it replaces is kept for reading at
`ports/_shared/evidence/iceandfire/pre-sdk/`.

## What it does

* `iceandfire:dragonegg_red` (the jar's item id) places `iceandfire:dragon_egg`.
* The egg **incubates only in a flame** — fire, soul fire, lava, magma block, campfire —
  for the jar's `eggBornTime` of **7,200 ticks** (6 minutes), one `age` step per 100
  ticks of flame; leaving the flame pauses it (it never cools).
* At its last age the egg becomes air and hatches one **`iceandfire:fire_dragon`**,
  tamed, owned by the **nearest player** (upstream's own promise: "The dragon will bond
  with the closest player") and homed where it hatched.
* A **wild** dragon (one `/summon`ed with no taming tags) looks for a player within its
  jar-configured `FOLLOW_RANGE` of 128 blocks, **flies at them** at the jar's flight
  speed, holds at the 2-block tackle reach — and **bites**: inside its inflated box it
  starts the jar's 35-tick bite animation and lands one hit for `ATTACK_DAMAGE` (1.0 at
  age 0) on the animation's 16th tick, once per bite.
* A **tamed** dragon does not hunt players and never bites; it hovers (**+0.08
  blocks/tick**, the jar's hover lift) and flies back when it strays more than 40 blocks
  from its home.
* A killed dragon is removed (before this it stayed in the world at 0 health).

## What it does not do

The other 21 entity types, the ice and lightning eggs and their frozen-egg block,
dragon roosts and their worldgen, the **breath weapon**, riding, ageing and growth, the
dragons' own loot tables, the other 11 egg colours, the scale/armour/weapon families, the
bestiary and the dragon's own model and texture are **not** in this port. `notes.md`
lists them with the reason each is out, and `FINDINGS.md` records what the jar contains.

The dragon **draws as its template's model** (a wandering trader): a mod's own mob model
is not data yet. Its AI is the port's own — the template's tick is cancelled in
`entity_tick_pre` — because a mob type's Part (`register_goals`/`register_behaviors`) ticks
inside the mob but its context has no position or velocity write, and the one per-entity
Part hook with a write host is for mod *general* entity types, which are not mobs and have
no renderer yet (`notes.md`, gap 2).

## Building and installing

```sh
cargo horizon-mod build          # target/horizon-mod/iceandfire.hmod
cargo horizon-mod check iceandfire.hmod
cargo horizon-mod fits  iceandfire.hmod --build 0a875da
```

The launcher installs a `.hmod` (content and code in one file) with `horizon-launcher
native-build . --trust-native-code`; there is no separate content package to install any
more, and no `.hnmod`.

## Checking it

```sh
cargo test --offline                  # 26 tests (18 flight/egg/glue, 8 bite)
cargo clippy --offline --all-targets -- -D warnings
ports/tools/launcher-game.sh build iceandfire    # a game with the package compiled in
eval "$(ports/tools/launcher-game.sh env iceandfire)" && tools/e2e.sh
```

`tools/e2e.sh` runs the in-game legs on a launcher-built game (The End arena, headless):
the bite and its tamed control, the egg item and block, the flame differential, the
7,200-tick hatch and the loot table, with a `SELFTEST=1` and a `CONTROL_ONLY=1` leg that
must fail. `VALIDATION.md` holds the counts and the evidence paths.
