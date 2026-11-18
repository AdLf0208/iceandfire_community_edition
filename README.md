# IceAndFire Community Edition — a Horizon port

**IceAndFire Community Edition** is a Minecraft mod. An unofficial fork of Ice And Fire. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [IAFEnvoy/IceAndFire-CE](https://github.com/IAFEnvoy/IceAndFire-CE) — licence **LGPL-3.0-or-later**.

| | |
|---|---|
| Upstream mod | IceAndFire Community Edition ([source](https://github.com/IAFEnvoy/IceAndFire-CE)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `iceandfire.hmod` (install it with the Horizon launcher) |
| Licence | **LGPL-3.0-or-later** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

### What it does not do (slice 1)

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

## Not yet ported

- **The API does not let a mod build a String NBT tag.** `Compound` and `Tag` are exported;
- **A mob type keeps no per-tick entity Part that can move the mob.** Measured at 0b14f8a:
- **No own model or texture of its own.** The dragon draws its template's (wandering
- **No growth/ageing, no hunger, no loot, no sounds.** The hatch sets the jar's age-0
- **The egg's progress is a block state, not a counter.** 72 `age` values of 100 ticks
- **Dispensers, `/setblock`, structure blocks and creative pick-block do not start the
- **Melee and riding are *available* at this API but not in this slice.** The old slice's
- The other 21 entity types, the roosts, the worldgen, the other egg colours and the

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `iceandfire.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

IceAndFire Community Edition was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **LGPL-3.0-or-later**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/IAFEnvoy/IceAndFire-CE

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run iceandfire` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/iceandfire/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
