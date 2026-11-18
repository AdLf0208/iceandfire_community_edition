# Ice and Fire Community Edition for Horizon — slice 1

One dragon, end to end: the **red fire dragon's egg** (the jar's own spawn mechanism),
its flame-gated incubation, the **tamed, owned fire dragon** it hatches, and that
dragon's **flight and pursuit**.

Ported from **Ice And Fire Community Edition** `2.0-beta.17` (`iceandfire`) by
**IAFEnvoy, xiaowu, uoay** — <https://github.com/IAFEnvoy/IceAndFire-CE>.
Licence **LGPL-3.0**; this port is a derivative under the same licence and its
complete source is published with it. See `LICENSE` and `LICENSE-NOTICE.md`.

This is the **SDK-format package** (mod API 0.2, Horizon build `0b14f8a`, 2026-11-18):
one flat `.hmod` holding the manifest, the Rust crate and the carried content. The
`.hnmod` it replaces is kept for reading at
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
  jar-configured `FOLLOW_RANGE` of 128 blocks and **flies at them** at the jar's flight
  speed, stopping at the 2-block tackle reach.
* A **tamed** dragon does not hunt players; it hovers (**+0.08 blocks/tick**, the jar's
  hover lift) and flies back when it strays more than 40 blocks from its home.

## What it does not do (slice 1)

The other 21 entity types, the ice and lightning eggs and their frozen-egg block,
dragon roosts and their worldgen, breath weapons and melee damage, riding, ageing and
growth, loot, the other 11 egg colours, the scale/armour/weapon families, the bestiary
and the dragon's own model and texture are **not** in this slice. `notes.md` lists them
with the reason each is out, and `FINDINGS.md` records what the jar contains.

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
cargo horizon-mod fits  iceandfire.hmod
```

The launcher installs a `.hmod` (content and code in one file) with `horizon-launcher
native-build . --trust-native-code`; there is no separate content package to install any
more, and no `.hnmod`.

## Checking it

```sh
cargo test --offline --locked                  # 18 tests (15 runtime, 3 glue)
cargo clippy --offline --all-targets -- -D warnings
ports/tools/build-slot.sh run --job sdk-iaf --port ports/iceandfire --step build -- \
    bash -c 'cd ports/iceandfire && cargo horizon-mod build'
```

`tools/check.sh` wraps the build/check/fits triple in a slot lease. `VALIDATION.md` holds
the counts and the evidence paths; there is **no in-game leg** for this migration (see its
"Not covered by this ladder").
