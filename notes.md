# Ice and Fire Community Edition — port notes (slice 1: one fire dragon)

SDK package at Horizon **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (game 26.3, build
2026-11-18), mixins **`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, mod API **0.2**, launcher
≥ 0.2.70. This file is the migration record of the `.hnmod` slice into the flat SDK package;
the retired layout is kept for reading at
`ports/_shared/evidence/iceandfire/pre-sdk/`.

| | |
|---|---|
| Mod | Ice And Fire Community Edition `2.0-beta.17` (`iceandfire-2.0-beta.17.jar`, **20,848,811 B**, 5,589 entries, sha256 `ee523494…5f8d90f0`) |
| Class / tier | `content` / **deep** (core, P0) — entities and AI |
| Licence | **LGPL-3.0** (jar `META-INF/neoforge.mods.toml`: `license = "LGPL-3.0"`; authors **IAFEnvoy, xiaowu, uoay**), **rank 1** — publishable under the same licence with the complete source; local-only until the maintainer approves (ARCHITECTURE.md §8) |
| Approach from `modpacks.md` | `deep`: content for the egg item/block and the entity type, agent-written code for incubation, the hatch and the dragon's flight/targeting |
| Slice | **the fire dragon, end to end** — egg → flame incubation → hatch (tamed, owned) → flight and pursuit; every other entity, the roosts and the gear are stated gaps |
| Package | one flat `.hmod`: `horizon-mod.toml` + `Cargo.toml` + `src/` + `assets/` + `data/` |

## Slice boundary

**In:** the jar's own spawn mechanism for its cheapest dragon —
`iceandfire:dragonegg_red` (the jar's item id) → `iceandfire:dragon_egg` block →
7,200 ticks of flame → one `iceandfire:fire_dragon`, tamed, owned by the nearest
player, homed at the egg → its hover flight (+0.08 blocks/tick) and a wild dragon's
player pursuit inside the jar's 128-block `FOLLOW_RANGE`, holding at the 2-block
tackle reach.

**Out (measured, in `FINDINGS.md` §3 and the gaps below):** the other 21 entity types,
the ice egg (needs the frozen-egg block entity and a 1/500 water conversion) and the
lightning egg (rain + sky visibility), roosts and their worldgen, breath weapons and
melee damage, riding, ageing/growth, loot, the other 11 egg colours, the scale/armour/
weapon families, the bestiary, and the dragon's own model and texture.

## Upstream behaviour read (public source; never decompiled)

Repo <https://github.com/IAFEnvoy/IceAndFire-CE>, commit
`a321dc4e8b2fee4aec7bf7723ffb4f4384c99fce` (2026-11-11; `gradle.properties`
`mod_version=2.2`, `minecraft_version=26.1.2` — the pack ships the 1.21.1
`2.0-beta.17`, which has **no tag** in the repository, so `main` was read and its
numbers re-checked against the jar's own lang/assets). Files read:
`entity/DragonEggEntity.java`, `entity/DragonBaseEntity.java`,
`entity/FireDragonEntity.java`, `entity/util/dragon/IafDragonLogic.java`,
`entity/ai/DragonAITargetGoal.java`, `entity/ai/DragonAITargetNonTamedGoal.java`,
`item/DragonEggItem.java`, `config/IafCommonConfig.java`,
`registry/IafDragonColors.java`. The clone is deleted after these notes; the jar stays
in `ports/_shared/licence-cache/` as the licence evidence.

Where each number comes from:

| Number | Value | Source |
|---|---|---|
| egg incubation | 7,200 ticks in flame | `IafCommonConfig.java:45` (`eggBornTime`), used at `DragonEggEntity.java:136` |
| hatch result | dragon, variant + random gender, `Hunger 50`, `setTame(true, true)`, owner = egg owner, egg discarded | `DragonEggEntity.java:143-159` |
| hatch bond | "the dragon will bond with the closest player" | `item.iceandfire.dragonegg.jei_desc` (jar lang) |
| fire dragon at age 0 | health **20**, attack damage **1**, movement speed **0.15** | `FireDragonEntity.java:48` + `DragonBaseEntity.refreshDirtyAttributes():950-967` with `maxHealth 500` (`IafCommonConfig.java:46`) |
| target search | **128** blocks (`FOLLOW_RANGE`) | `IafCommonConfig.java:68`, `DragonAITargetNonTamedGoal.java:44-51` |
| wild-only targeting | `canUse` returns false when tamed | `DragonAITargetNonTamedGoal.java:29-34` |
| home leash | **40** blocks | `IafCommonConfig.java:69` |
| flight ceiling | **256** (`maxFlight`) | `IafCommonConfig.java:62`, `isBeyondHeight():1443-1448` |
| hover lift | **+0.08** blocks/tick | `IafDragonLogic.java:143` |
| flight speed | `MOVEMENT_SPEED × (5.2 + 1.5t) × 0.1` per tick against 0.9 damping → **0.78** blocks/tick at age 0 | `DragonBaseEntity.travel():2090-2165` |
| tackle reach | boxes grown by **2** blocks | `IafDragonLogic.java:112-118` |
| flame test | upstream `BlockUtil.isBurning` (uranus library, not in the repo) | see deviations |

## Migration to the SDK format

The `.hnmod` container was retired (mod-API merge #16941). The package is now one flat
`.hmod` built by the launcher against the SDK build it pins. Nothing about the *logic*
changed: `src/runtime.rs` is the old `mixins/iceandfire/runtime.rs` **byte-identical**
(sha256 `b28ea3dc89cfeb7177e7d4d2e542cb2476a01c306ec72cc2a2db2dd3afb02279`, `cmp` in
`evidence/iceandfire/runtime-identity.log`), with its 15 tests. Three tests were added
in `src/glue.rs` for the two new pure helpers (the `age` parse and the UUID form).

### File map

| Old (`.hnmod`) | New (`.hmod`) | What changed |
|---|---|---|
| `mixins/iceandfire/horizon-mixin.toml` | `horizon-mod.toml` | `id`/`name`/`version = 0.2.0`/`api = "0.2"`/`side`/`capabilities`, `[native]` with the three exact pins. No `[compatibility]`, no `[[module]]`, no `[[injection]]` |
| `Cargo.toml` (standalone runtime lib) | `Cargo.toml` (`horizon-mod-api` path dep into `.horizon/sdk/`) | the crate now compiles *against* the API; the runtime is a module of it |
| `mixins/iceandfire/runtime.rs` | `src/runtime.rs` | **byte-identical** |
| `mixins/iceandfire/support.rs` | `src/glue.rs` | rewritten: host calls instead of `Game` methods; the `//!` header is allowed now (it is a real module, not a planner module) |
| `place_block.rs`, `tick_block.rs`, `level_tick_post.rs`, `entity_tick_listened.rs`, `entity_tick_pre.rs` | `src/lib.rs` (one `impl Hooks` + two Part impls) | six helper files and six injections became four methods and two Parts |
| `embedded_content.rs` + `data/iceandfire-content.hmod` | *(deleted)* | the flat `.hmod` carries `assets/` and `data/` itself: the AE2 embedding workaround and `native_data!` are gone |
| `content/{horizon-mod.toml,assets,data}` | `assets/`, `data/` | merged into the package root; the item's `place_block` field is now `block` (API 0.2 schema) |
| `rust-toolchain.toml` | *(deleted)* | the pinned build decides the toolchain (`sdk.json`: rust 1.99.1) |
| `compatibility.json` | *(deleted)* | `[native] horizon`/`mixins` are the same statement inside the manifest |
| `iceandfire.hnmod` | `iceandfire.hmod` | one file, content + code |
| `tools/{horizon.py,package.py}` | `tools/check.sh` | staging and packaging are the launcher's job now; the tool just leases a slot and runs build/check/fits |

### Helper → hook table

| Old injection (hook API) | New surface | Kept / gap |
|---|---|---|
| `place_block` (14, HEAD, any block placed) | `ItemPart::on_block_placed_by` on `iceandfire:dragonegg_red` (`BlockItemExtension.onBlockPlacedBy`) | **kept, finer**: the moment is now the item's own, not every placement in the game; the egg is remembered in `ARMING` and armed on the level's next tick (see *Deviations*) |
| `tick_block` (34, HEAD on `GameLevel::native_tick_block`) | `BlockPart::tick_block` on `iceandfire:dragon_egg` (`BlockState.tick`), `&mut dyn LevelWriteHost` + `Flow::Cancel` | **kept**: the part is state-keyed to our block, so the name test is gone; `handled` is `Flow::Cancel` |
| `level_tick_post` (23, HEAD) | `Hooks::level_tick_post(&self, HostSide, LevelTickPostEvent { level })` | **kept**, plus it now arms the eggs placed that tick |
| `entity_tick_listened` (146, HEAD, `listened`) | `Hooks::entity_tick_listened` → `event.listened = true` | **kept**: same hook, same name, typed event |
| `entity_tick_pre` (146, HEAD, `canceled`) | `Hooks::entity_tick_pre(host, &mut EntityTickPreEvent { dimension, entity, kind, cancelled }) -> Flow::Cancel` | **kept**: same hook, typed event; no `&str` of the dimension is needed to move the mob (`EntityRef::teleport_to`) |
| `Game::level_players` (in-crate read) | `LevelRef::online_players(host)` → `Vec<PlayerRef>` | **kept**; gap 1 of the old slice (LAN-only `native_server_players`) is not needed |
| `native_teleport_mob` (37) | `EntityRef::teleport_to(host, pos, rotation)` | **kept** |
| `native_summon` (59) | `ServerHost::summon(dimension, kind, pos, nbt)` | **kept** |
| `native_entity_nbt` (386) | `EntityRef::nbt(host)` | **kept** (read); the write is the `summon` payload |
| `native_set_block_property` (24) | `LevelWriteHost::set_block_property(pos, key, value, flags)` | **kept** |
| `behaviours.set_block(air, UPDATE_ALL)` | `LevelWriteHost::remove_block(pos, false)` | **kept**, closer to vanilla (`Level.removeBlock`) |
| `ticks.schedule_block(.., TickPriority::Normal)` | `LevelWriteHost::schedule_block_tick(pos, block, delay)` | **kept** (no priority argument) |
| `embedded_content` (`content_root` HEAD + `native_data!`) | — | **not needed**: the package carries its own content |
| `Game::with_level` (the old place-time schedule) | `LevelRef::block_tick(host, pos)` from `level_tick_post` | **kept** as the arming step |

### Manifest mapping

```toml
id = "iceandfire"           # the resource namespace, and the only identity
version = "0.2.0"           # the new version; #925 was 0.1.0
api = "0.2"
side = "both"
capabilities = ["registries"]
[native]
game = "26.3"
horizon = "0b14f8acfd97fa2c750e42795249dcc981178ca4"
mixins  = "e59da9094f0c2fdda6c627fe9eff22dec6a906f0"
```

### Build, check, fits

```sh
ports/tools/build-slot.sh run --job sdk-iaf --port ports/iceandfire --step build -- \
    bash -c 'cd ports/iceandfire && cargo horizon-mod build'
cargo horizon-mod check iceandfire.hmod      # package check (does not compile the crate)
cargo horizon-mod fits  iceandfire.hmod      # "0b14f8a  made for"
```

### Traps the next huge port must know

1. **The old `mixins/` folder is refused by name.** `cargo horizon-mod build` walks the
   project and stops at `unexpected package path: mixins/iceandfire/runtime.rs`. Move the
   old folder out (keep it under `ports/_shared/evidence/<mod>/pre-sdk/`) before the first
   build; other extras (`tools/`, `notes.md`, `FINDINGS.md`) are tolerated but not packed.
2. **`check` validates the package, not the code** ("native source: unsandboxed; not
   executed by the registry check"). Compile against the SDK yourself
   (`cargo test` / `cargo clippy --all-targets`) — that is what catches an API mistake.
3. **Host methods are on the *handles* now.** `host.entity_nbt(..)` is the hidden backend;
   a mod calls `entity.nbt(host)`, `entity.teleport_to(host, ..)`, `level.online_players(host)`,
   `living.alive(host)`. The compiler's "method not found on `&mut dyn ServerHost`" means
   *use the handle*, not *import a trait*.
4. **A Part is not a stub you can put anywhere.** A Part needs `#[mod_api::horizon_part]`
   (it generates the subscription table) and the hook's id must be one the Part kind
   offers: `ItemPart`/`BlockPart` here, `EntityPart` for mobs (goals, behaviors) — and see
   gap 2 for what an `EntityPart` cannot do.
5. **`cargo fmt --all` formats the fetched SDK too.** Format `src/*.rs` (`rustfmt src/*.rs`).
6. **Content schema moved**: an item places a block with `"block"`, not `"place_block"`
   (`unknown field 'place_block'` is the error).
7. The package is deterministic: two builds give the same sha256 (evidence log).

## The four source roles (SDK layout)

* `src/runtime.rs` — pure: flame set, egg stepping, target choice, flight, UUID formatting
  (15 tests, **byte-identical** to the `.hnmod` slice).
* `src/glue.rs` — glue: the arming list, the hatch queue, the block reads/writes, the summons,
  and two small tested helpers (the serialized-state `age` parse, the UUID's 4-int form).
* `src/lib.rs` — the one `#[mod_api::horizon_mod] impl Hooks` (register, entity tick listen,
  entity tick, level tick) and the two Parts.
* `horizon-mod.toml` — the manifest; `assets/` + `data/` — the carried content.

## Gaps (stated, not faked)

1. **The API does not let a mod build a String NBT tag.** `Compound` and `Tag` are exported;
   `JavaString` is not, so `Tag::String` cannot be constructed. The owner is therefore
   written as vanilla's own UUID form (`iceandfire:Owner` as the 4-int array `NbtUtils.loadUUID`
   reads) instead of the hyphenated string the `.hnmod` slice wrote. Issue shape: *"export
   `JavaString` (or a `Tag::string(&str)` constructor) so a port can write the NBT shapes
   vanilla and other mods read as strings."*
2. **A mob type keeps no per-tick entity Part that can move the mob.** Measured at 0b14f8a:
   `EntityPart` on a mob type offers `register_goals`/`register_behaviors` (custom `Goal`s and
   brain `Behavior`s, ticked inside the mob's own AI step) — but their contexts (`GoalContext`,
   `BehaviorCx`) expose the mob handle, a *read* host, navigation/look/jump and the brain's
   memories, and **no position or velocity write**, so the port's teleport-based flight cannot
   live there. The one per-entity Part hook with a write host, `general_entity_tick`, is told
   only for mod **general** entity types (`declare_general_entity`), which are not mobs, keep
   no health/attributes and **have no renderer yet** ("a mod's general entity types will
   register the same" — `register_entity_renderers` doc). So the dragon's tick stays
   `Hooks::entity_tick_pre`, scoped by `kind`; a Part that owns the AI is possible the day
   either the goal context gets a write host or general types get renderers.
3. **No own model or texture of its own.** The dragon draws its template's (wandering
   trader) model; own models are "not yet as data" (`docs/mods.md:367-369`) and need
   `native_mob_submit` + model layers (next slice). Unchanged by the migration.
4. **No growth/ageing, no hunger, no loot, no sounds.** The hatch sets the jar's age-0
   attributes (health 20); the 24,000-tick age step, `Hunger 50`, the dragon loot tables and
   the `EGG_HATCH`/`FIRE_EXTINGUISH` sounds are out.
5. **The egg's progress is a block state, not a counter.** 72 `age` values of 100 ticks
   (7,200 ticks, the jar's own total); a flame is sampled once per 100 ticks, and the first
   look is 20 ticks after placement.
6. **Dispensers, `/setblock`, structure blocks and creative pick-block do not start the
   incubation**: only the egg item's placement is heard (`ItemPart::on_block_placed_by`), and
   its hook carries no level for a dispenser (no placer). An egg placed by any other route
   stays inert until something schedules it.
7. **Melee and riding are *available* at this API but not in this slice.** The old slice's
   gap ("a mob cannot deal damage and cannot ride") **no longer holds**: `EntityRef::hurt(host,
   damage_type, amount, direct, causing)` and `EntityRef::start_riding(host, vehicle, force)`
   exist and type-check against a hook's `ServerHost` (probe:
   `ports/_shared/evidence/iceandfire/api-probe/`, `cargo check` exit 0). They are the next
   slice (bite/breath, rider controls), not this migration's scope.
8. The other 21 entity types, the roosts, the worldgen, the other egg colours and the
   gear families are not started (see "Out").
9. **No in-game leg in this migration.** The SDK format is run by `horizon-launcher`
   (`cargo horizon-mod run` answers *"code mods run through the launcher's native build"*),
   which is not installed here; the `.hnmod` e2e at the old pin (egg ages 0 on stone vs 3 on
   magma, hatch to `Tamed 1b` + `Owner` + health 20.0, dragon rises 68.3 → 71.0 in 30 ticks,
   wild dragon closes 16.15 → 1.35) is the last in-game evidence and is quoted in
   `VALIDATION.md`. `check`+`fits`+`cargo test`+`clippy` are this migration's evidence.

## Deviations (design choices, not defects)

* **Incubation source.** Upstream's `BlockUtil.isBurning` lives in the `uranus` library,
  which is not in the mod's repository, so the port's flame set is stated: fire, soul
  fire, lava, magma block, campfire, soul campfire (`runtime::FLAME_BLOCKS`).
* **Owner at hatch.** Upstream stores the *placer* on the egg entity; a block cannot
  hold a UUID, so the port bonds the hatchling to the **nearest player**, which is what
  the mod's own item description promises. In single player the two are the same player.
  The nearest *player to the egg* is now measured from the egg (the `.hnmod` slice measured
  from the world origin; see "Fixed in the migration").
* **Owner tag form.** `iceandfire:Owner` is a 4-int array, vanilla's UUID form, because the
  API exports no `JavaString` (gap 1).
* **Flight model.** The port moves the dragon by teleport (0.78 blocks/tick cruise,
  0.08/tick hover lift, ceiling 256) rather than reproducing vanilla's `moveRelative`
  integration; acceleration, drag and pitch are not modelled.
* **The egg block is the port's own** (`iceandfire:dragon_egg`, template
  `minecraft:stone` with the vanilla dragon-egg model and shape). Upstream's egg is an
  *entity*; a block was chosen so progress is saved in the world with no block entity.
* **Arming is one level tick late.** The placement's hook has no scheduling call
  (`ServerHost` exposes none; `LevelRef::block_tick` runs a scheduled tick, it does not queue
  one), so the position is remembered and the level's next tick arms it with
  `LevelRef::block_tick`. The chain that follows is the level's scheduled ticks, exactly as
  before.
* **The `age` is read from the state's serialized form.** A block part gets no property
  reader (`BlockState::property` wants a metadata resource a part is not given), so
  `glue::age_of` parses `iceandfire:dragon_egg[age=N]`; a value the parser cannot read is
  age 0.
* **No flame visual or cracks** on the egg; every age state draws the same model.
* **Yaw** is not written (the dragon drifts without turning to face its target).

## Upstream defects fixed

None found that this slice could prove. Two candidate inconsistencies were checked and
left as they are, because they are design choices, not broken behaviour:

* Upstream's fire ticker plays no sound on each age step and plays `FIRE_EXTINGUISH`
  only at the hatch (`DragonEggEntity.java:220-224`); the port also stays silent (gap 4).
* The egg item's description promises a bond with the closest player while the code
  stores the placer — the port follows the description (deviation "Owner at hatch").

Worth an upstream report: the item description vs. `onPlayerPlace` wording (cosmetic);
not filed.

### Fixed in the migration (the port's own bug, not the jar's)

**The hatch's "closest player" was measured from the world origin.** The `.hnmod` slice's
`nearest_player` minimised `x² + y² + z²` — a distance to `(0,0,0)` — so in a world with two
or more players it could bond the dragon to the wrong one. The glue now measures from the
egg's own position (`glue::nearest_player`), which is what
`item.iceandfire.dragonegg.jei_desc` promises. A user notices in multiplayer: the dragon
bonds to the player standing by the egg, never to one who happens to be near spawn. It is
worth no upstream report (the jar does not have this bug).

## Verification

See `VALIDATION.md` and `ports/_shared/evidence/iceandfire/`. One deliverable:
`ports/iceandfire/iceandfire.hmod` (**41,200 B**, sha256
`b4023b521155e131ae7f17005c79fab8c5735a0ccaf86e2702692fdb59712109`, 21 entries, deterministic
across two builds), Licence files inside; `check` prints
`native-mod, format native-1/0b14f8a…/e59da90…, for 26.3`, exit 0; `fits` prints
`0b14f8a  made for`, exit 0; `cargo test` **18 passed** (15 runtime byte-identical + 3 glue).

## Changelog for the resubmission

> Supersedes #925, closed because Horizon retired `.hnmod` (#17992). Changes: flat SDK
> package with `api = "0.2"` and `[native]` pinned to `0b14f8a`; hooks rewritten as
> `impl mod_api::Hooks` methods against typed hosts/events (13 hooks/helpers mapped, 3
> recorded as gaps); `runtime.rs` and its tests unchanged.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port
