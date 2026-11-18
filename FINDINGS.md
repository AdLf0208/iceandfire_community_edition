# Ice and Fire Community Edition — findings (jar inventory + upstream behaviour)

Pinned artefact: `ports/_shared/licence-cache/iceandfire-2.0-beta.17.jar`
(**20,848,811 B**, 5,589 entries, sha256
`ee52349445615417e69ab64cf15dbb96b9332a9117225119d281695c5f8d90f0`), read from the pack.

Licence, read from the jar's own `META-INF/neoforge.mods.toml`:

```
license = "LGPL-3.0"
modId = "iceandfire"  version = "2.0-beta.17"
displayName = "Ice And Fire Community Edition"
authors = "IAFEnvoy, xiaowu, uoay"   credits = "Alexthe666"
description = "An unofficial fork of Ice And Fire."
```

The jar ships **no** `LICENSE` entry (`unzip -l | grep -i license` → none); the
licence text and the notice come from the upstream repository's `LICENSE`
(<https://github.com/IAFEnvoy/IceAndFire-CE>, `Copyright (c) 2024 IAFEnvoy`,
LGPL-3.0 plus the GPL-3.0 it incorporates), copied verbatim into
`LICENSE`.

## 1. What the artefact contains (census)

| Family | Count in the jar |
|---|---|
| entries | 5,589 |
| blockstates | 117 |
| block models | 145 |
| item models | 533 |
| item textures | 384 |
| recipes (`data/iceandfire/recipe/`) | 399 |
| advancements | 345 |
| block loot tables | 100 |
| entity loot tables | 27 (9 of them dragons) |
| bestiary lang chapters | 96 per language, 7 languages |
| compiled classes (`com/iafenvoy/iceandfire/…`) | the mod's code |
| entity types registered (`IafEntities`) | **22** |
| custom `resourcepacks/iaf_legacy/…` item textures | 303 |

Entity types (from the lang keys `entity.iceandfire.*` and the upstream
registry): `fire_dragon`, `ice_dragon`, `lightning_dragon`, `dragon_egg`,
`dragon_multipart`, `fire_dragon_charge`, `ice_dragon_charge`,
`lightning_dragon_charge`, `dragon_arrow`, `dragon_skull`, `amphithere`,
`cockatrice`, `cyclops`, `death_worm`, `dread_beast`, `dread_ghoul`,
`dread_horse`, `dread_knight`, `dread_lich`, `dread_scuttler`, `dread_thrall`,
`ghost`, `gorgon`, `hippocampus`, `hippogryph`, `hydra`, `pixie`,
`sea_serpent`, `siren`, `stymphalian_bird`, `troll`, `myrmex` families.

Dragon eggs are **items**, one per colour (12: `dragonegg_red`,
`dragonegg_green`, `dragonegg_bronze`, `dragonegg_gray`, `dragonegg_black`,
`dragonegg_white`, `dragonegg_sapphire`, `dragonegg_blue`, `dragonegg_silver`,
`dragonegg_copper`, `dragonegg_electric`, `dragonegg_amethyst`), with
`assets/iceandfire/models/item/dragonegg_<colour>.json` on
`item/generated` and a 16×16 texture each. The **fire** dragon's colours are
red, green and bronze (`IafDragonColors.java:14-16`); red is the first.

There is **no** dragon-egg block model or blockstate in the jar: upstream's egg
is an **entity**, not a block (`DragonEggEntity`), and the only egg-shaped block
is `egginice` (the frozen ice-dragon egg).

## 2. Upstream behaviour read (public source, not decompiled code)

Repo <https://github.com/IAFEnvoy/IceAndFire-CE>, clone at
`ports/_shared/upstream/iceandfire`, commit
`a321dc4e8b2fee4aec7bf7723ffb4f4384c99fce` (2026-11-11, `gradle.properties`
`mod_version=2.2`, `minecraft_version=26.1.2`). The pack ships
`2.0-beta.17` for 1.21.1 and the repository has **no tag** for it
(`git ls-remote --tags` → 1.0 … 1.2.2 only), so the head of `main` was read;
the dragon/egg numbers below were re-checked against the jar's own lang and
assets where the two could differ. Areas read: `entity/DragonEggEntity.java`,
`entity/DragonBaseEntity.java`, `entity/FireDragonEntity.java`,
`entity/util/dragon/IafDragonLogic.java`, `entity/ai/DragonAITargetGoal.java`,
`entity/ai/DragonAITargetNonTamedGoal.java`, `item/DragonEggItem.java`,
`config/IafCommonConfig.java`, `registry/IafDragonColors.java`.

### 2.1 The egg → hatch chain (upstream's own spawn mechanism)

`DragonEggItem.useOn` (`DragonEggItem.java:51-64`) spawns a
`DragonEggEntity` above the clicked face and records the **player who placed
it** as the egg's owner (`entity.onPlayerPlace(context.getPlayer())`).
`DragonEggEntity.updateEggCondition` (`DragonEggEntity.java:134-160`) runs each
tick:

* `hatched = getDragonAge() > eggBornTime`, **`eggBornTime` default 7200**
  ticks (`IafCommonConfig.java:45`);
* the fire ticker (`DragonEggEntity.java:219-225`) increments `DragonAge` by 1
  **only while the block at the egg's position is burning**
  (`BlockUtil.isBurning`, a helper of the `uranus` library — not in this
  repository, so the port states its own flame set, §4.1);
* on hatch: the block at the egg is set to air, a dragon is created, its
  variant and gender set, `setHunger(50)`, **`setTame(true, true)`**, owner =
  the egg's owner, the egg discarded, `EGG_HATCH` sound played
  (`DragonEggEntity.java:143-159`).

So a hatched dragon is **tamed and owned by the player who placed the egg** —
no feeding is needed. `creative_dragon_meal` is the other taming path
(`DragonBaseEntity.java:1155-1167`: `setTame`, `tame(player)`, +20 hunger,
heal, one item used).

### 2.2 Dragon numbers (fire dragon, age 0)

`FireDragonEntity.java:48` passes
`(minDamage 1, maxDamage 1 + attackDamage, minHealth maxHealth*0.04,
maxHealth, minSpeed 0.15, maxSpeed 0.4)` with the config defaults
`attackDamage 17`, `maxHealth 500` (`IafCommonConfig.java:46-47,61`).
`refreshDirtyAttributes` (`DragonBaseEntity.java:950-967`) interpolates over
125 days: at age 0 that is **health 20, attack damage 1, movement speed 0.15,
follow range `min(2048, targetSearchLength)` = 128** (`targetSearchLength`
default 128, `IafCommonConfig.java:68`).

Other numbers the port uses, each from the cited line:

| Fact | Value | Source |
|---|---|---|
| egg incubation | 7,200 ticks in flame | `IafCommonConfig.java:45` |
| dragon target search | 128 blocks (`FOLLOW_RANGE`) | `IafCommonConfig.java:68`, `DragonAITargetNonTamedGoal.java:48-51` |
| wild dragon targets players | priority 5, `canUse` false when tamed | `DragonAITargetNonTamedGoal.java:29-46` |
| home leash | 40 blocks (`wanderFromHomeDistance`) | `IafCommonConfig.java:69` |
| flight ceiling | 256 (`maxFlight`), also the level height | `IafCommonConfig.java:62`, `DragonBaseEntity.isBeyondHeight():1443-1448` |
| hover lift | **+0.08 blocks/tick** (+0.12 in water) | `IafDragonLogic.java:143` |
| flight trigger | random 1/1500 per tick, or target 5 blocks above/below, or in water | `DragonBaseEntity.java:116`, `IafDragonLogic.java:189-196` |
| flight speed | `MOVEMENT_SPEED * airSpeedModifier(5.2 + 1.5·t) * 0.1` per tick, velocity × 0.9 per tick (`travel`) | `DragonBaseEntity.java:2090-2165` |
| tackle contact | boxes grown by 2 blocks intersect | `IafDragonLogic.java:112-118` |
| melee bite | `ATTACK_DAMAGE` while the bite animation is at tick 15-25 | `IafDragonLogic.java:403-425` |
| egg hatch hunger | 50 | `DragonEggEntity.java:148` |

At age 0 the flight formula's terminal speed is
`0.15 · 5.2 · 0.1 / (1 − 0.9) = 0.78` blocks/tick; the port uses that number.

## 3. The slice chosen from the artefact (and why)

The brief allows one dragon end to end. The jar supports the **fire dragon**
most cheaply, because its egg's incubation test is a single block read
(`isBurning`), while the ice egg needs the `egginice` block entity and a 1/500
water conversion (`DragonEggEntity.java:226-238`) and the lightning egg needs
rain + sky visibility (`:239-249`).

**In (slice 1):** the red fire-dragon egg item, its incubation block, the
7,200-tick hatch, the tamed+owned dragon it produces, and that dragon's
flight/targeting behaviour (hover climb at +0.08/tick, pursuit of the nearest
player inside 128 blocks, stop at the 2-block tackle reach), plus a control
that a non-dragon entity is untouched.

**Out (stated gaps):** the other 21 entity types, the ice and lightning eggs
and their blocks, roosts/worldgen, breath weapons and melee damage, riding,
growth/ageing, loot, the 12-colour egg family (1 carried), the gear families,
the bestiary, and the dragon's own model/texture (the pin cannot render a mod
mob's own model or texture as data, so the slice's dragon draws its template's
model — `native_mob_submit` + model layers is the next slice's headline).

## 4. Platform facts measured against the pin (`1d63e0a`)

1. **The pin's living entity types** copy one of `minecraft:villager`,
   `minecraft:wandering_trader`, `minecraft:zombie`
   (`crates/horizon-mods/src/content.rs:396-400`). A type's own model, texture
   and goals are **not** data (`docs/mods.md:367-369`), and goal replacement
   is the draft `[api]`/`register_goals` path, which the pin's mixins parser
   rejects (ARCHITECTURE.md §3, `[api]` gate). So this port's dragon is a
   `wandering_trader`-template mob whose **tick the port cancels**
   (`entity_tick_pre`, `canceled = true`) and whose motion the port writes with
   `native_teleport_mob`: the port is the AI, the template contributes the
   hitbox, the saving and the drawn model.
2. `native_server_players()` is **empty for the single-player host** — it reads
   the LAN guests (`game/native_server.rs:313-322`) — so a server hook cannot
   find the local player through the mod surface. The port reads the in-crate
   `Game::level_players()` (`game/natural_spawning.rs:691`), whose `LevelPlayer`
   carries the host and the guests; recorded as an API gap in `notes.md`.
3. A block that self-schedules is the only way to tick without a hook that can
   schedule: `native_place_block` (API 14) has no `ticks` handle, so the first
   incubation tick is scheduled from it with the in-crate
   `Game::with_level(...)` (`ticks.schedule_block`), the pattern `utilitarian`
   uses for snad.
4. `minecraft:dragon_egg` is a `DragonEggBlock` **and a `FallingBlock`**
   (`block-behaviours.json`), so it is a poor template for a placed egg (it
   falls off a fire block and teleports when punched). The port's egg block
   copies `minecraft:stone` and declares the egg's shape from the vanilla
   `block/dragon_egg` model, which is 6 boxes — a small, static, non-falling
   block.

## 5. Re-measured at the SDK build (`0b14f8a`, 2026-11-18)

§4 is the **`.hnmod` pin's** (`1d63e0a`) measurement and stays as the record of why the
0.1.0 slice took the shape it took. Four of those facts changed with the mod API, and the
migration re-checked them instead of repeating them:

| §4 fact | At `0b14f8a` |
|---|---|
| "a mob cannot deal damage and cannot ride" | **no longer true**: `EntityRef::hurt(host, damage_type, amount, direct, causing)` and `EntityRef::start_riding(host, vehicle, force)` exist and type-check on a hook's `ServerHost` (`ports/_shared/evidence/iceandfire/api-probe.md`). Still out of this slice, as *unbuilt*, not *unavailable* |
| "goal replacement is the draft `[api]` path" | `EntityPart::register_goals` and `register_behaviors` are real API (`mod_api::{Goal, GoalFlags, GoalContext, Behavior, BehaviorRegistrar}`), ticked inside the mob's own AI step — but a goal's context has **no position/velocity write**, so the port's teleport flight cannot move into one |
| "`native_server_players()` is empty for the single-player host" | the mod surface now has `LevelRef::online_players(host)` (`ServerLevel.players`), which the port uses for the hatch's owner and the wild dragon's target |
| "the first incubation tick is scheduled from the placement through `Game::with_level`" | a placement's hook has **no scheduling call**: the placement is remembered and the level's next tick arms it with `LevelRef::block_tick(host, pos)`, whose block part then schedules the chain |

One new limit was found: the API exports `Compound` and `Tag` but **not `JavaString`**, so
a mod cannot build a `Tag::String`; the owner UUID is written in vanilla's own 4-int array
form (`notes.md`, gap 1).
