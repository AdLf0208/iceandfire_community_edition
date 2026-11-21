//! Game-side glue for the fire dragon: the egg block's chain, the hatch queue and the
//! dragon's flight.
//!
//! Every rule lives in [`crate::runtime`]; this file only reads the world, calls it and
//! writes the result back. The facts are cited in `../FINDINGS.md` §2 and `notes.md`:
//!
//! * the egg incubates while the block it stands in or on is a flame and hatches after
//!   `eggBornTime` 7,200 ticks (`DragonEggEntity.java:134-225`);
//! * the hatched dragon is tamed and owned by "the closest player"
//!   (`item.iceandfire.dragonegg.jei_desc`), home at the egg;
//! * a wild dragon looks for a player within `FOLLOW_RANGE` (128) and flies at them; a
//!   tamed one does not (`DragonAITargetNonTamedGoal.java`);
//! * movement is the jar's hover lift and flight formula (`IafDragonLogic:143`,
//!   `DragonBaseEntity:2090-2165`).
//!
//! The pin gives a mod no goal surface for its own mob type's tick that can also move
//! the mob, and its own model is not data yet, so the dragon is a `wandering_trader`
//! template mob whose tick the port cancels in `entity_tick_pre`: the port is its AI,
//! the template gives the hitbox, saving and drawn model. See `notes.md` §Migration.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use mod_api::{
    BlockUpdateFlags, Compound, EntityRef, EntityTickPreEvent, Flow, LevelHost, LevelRef,
    LevelWriteHost, OnBlockPlacedByEvent, PlayerRef, ServerHost, Tag, TickBlockEvent,
};

use crate::runtime::{
    AGE0_ATTACK_DAMAGE, Bite, DRAGON_FLIGHT_CEILING, EGG_SCHEDULE_DELAY, aggro_inflate,
    choose_target, dragon_step, egg_step, grown, intersects, is_flame, within_reach,
};

/// The incubated egg block this port's content adds.
pub const EGG_BLOCK: &str = "iceandfire:dragon_egg";
/// The red dragon egg item that places it (the jar's `iceandfire:dragonegg_red`).
pub const EGG_ITEM: &str = "iceandfire:dragonegg_red";
/// The fire dragon entity type (`entity.iceandfire.fire_dragon`).
pub const DRAGON: &str = "iceandfire:fire_dragon";
/// The `age` state value of an egg that has just been placed.
const AGE_PROPERTY: &str = "age";
/// The first look at a freshly placed egg comes this many ticks after the placement, as
/// it did before the migration: the placement's hook cannot schedule a block tick, so the
/// level's next tick arms it and waits one second.
const FIRST_TICK_DELAY: i32 = 20;
/// The tamed flag the hatch writes. The jar saves `Color`, `DragonAge` and `OwnerUUID` on
/// its dragon (`DragonEggEntity.java:71-73`); the port keeps the bond as tags of its own,
/// because it has no mob entity type of its own to declare a field on.
const TAMED_TAG: &str = "iceandfire:Tamed";
const OWNER_TAG: &str = "iceandfire:Owner";
const HOME_TAGS: [&str; 3] = ["iceandfire:HomeX", "iceandfire:HomeY", "iceandfire:HomeZ"];

/// An egg that reached its last age: it is air, and the dragon is summoned at the next
/// level tick (a block part's tick holds the level, not the game).
struct Hatch {
    dimension: &'static str,
    at: [f64; 3],
}

/// Eggs whose placement is waiting for the level's next tick to start their chain.
///
/// A placement's hook holds the server, which cannot schedule a block tick; the level's
/// own tick can (`LevelRef::block_tick`), and this list carries the position across that
/// one tick. An entry is dropped when it is armed, or when the egg is gone.
static ARMING: Mutex<Vec<(&'static str, [i32; 3])>> = Mutex::new(Vec::new());
static HATCHES: Mutex<Vec<Hatch>> = Mutex::new(Vec::new());

/// A red dragon egg was placed: remember where, so the level's next tick starts its
/// incubation, with the dimension the placer's level names.
pub(crate) fn egg_placed(host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
    let Some(dimension) = event
        .placer
        .and_then(|placer| placer.entity().level(host))
        .map(LevelRef::dimension_id)
    else {
        // A dispenser has no placer and its level is not in the event: the egg stays
        // inert, which is the pre-migration port's own limit (`notes.md`, gap 6).
        return;
    };
    let mut arming = ARMING.lock().expect("iceandfire arming lock poisoned");
    if arming.len() > 1024 {
        arming.clear();
    }
    arming.push((dimension, [event.pos[0], event.pos[1], event.pos[2]]));
}

/// One scheduled tick of an egg: the first look only schedules the next one, every later
/// look ages the egg one step in flame, and the last one hatches it.
pub(crate) fn egg_tick(level: &mut dyn LevelWriteHost, event: &mut TickBlockEvent) -> Flow {
    let at = [event.pos.x, event.pos.y, event.pos.z];
    let dimension = event.dimension.key().id();
    {
        let mut arming = ARMING.lock().expect("iceandfire arming lock poisoned");
        if let Some(first) = arming.iter().position(|entry| *entry == (dimension, at)) {
            arming.swap_remove(first);
            // The placement's own tick: wait a second before the first look, as the
            // pre-migration port did.
            level.schedule_block_tick(at, event.state.block(), FIRST_TICK_DELAY);
            return Flow::Cancel;
        }
    }
    let age = age_of(&level.state_string(event.state).unwrap_or_default());
    let burning = flame_at(level, at) || flame_at(level, [at[0], at[1] - 1, at[2]]);
    let step = egg_step(age, burning);
    if step.hatch {
        // The egg is replaced by air, and the dragon is summoned at the next level tick
        // (the jar's `DragonEggEntity.java:144` sets the block to air).
        level.remove_block(at, false);
        let mut pending = HATCHES
            .lock()
            .expect("iceandfire hatch queue lock poisoned");
        pending.push(Hatch {
            dimension,
            at: [
                f64::from(at[0]) + 0.5,
                f64::from(at[1]),
                f64::from(at[2]) + 0.5,
            ],
        });
        return Flow::Cancel;
    }
    if step.age != age {
        level.set_block_property(
            at,
            AGE_PROPERTY,
            &step.age.to_string(),
            BlockUpdateFlags::CLIENTS.bits(),
        );
    }
    if step.again {
        level.schedule_block_tick(at, event.state.block(), EGG_SCHEDULE_DELAY);
    }
    Flow::Cancel
}

/// A level finished its tick: arm the eggs placed in it, then summon the dragons whose
/// eggs hatched in it, each bonded to the closest player.
pub(crate) fn level_tick(host: &mut dyn ServerHost, level: LevelRef) {
    let dimension = level.dimension_id();
    let arming: Vec<[i32; 3]> = {
        let mut arming = ARMING.lock().expect("iceandfire arming lock poisoned");
        let mut mine = Vec::new();
        arming.retain(|(at_dimension, at)| {
            let is_mine = *at_dimension == dimension;
            if is_mine {
                mine.push(*at);
            }
            !is_mine
        });
        mine
    };
    for at in arming {
        // The egg block's part runs now and schedules the chain. A position whose chunk is
        // not ticking is dropped: the egg stays inert, as a `/setblock` egg does.
        level.block_tick(host, at);
    }

    let pending: Vec<Hatch> = {
        let mut queue = HATCHES
            .lock()
            .expect("iceandfire hatch queue lock poisoned");
        let mut mine = Vec::new();
        queue.retain(|hatch| {
            let is_mine = hatch.dimension == dimension;
            if is_mine {
                mine.push(Hatch {
                    dimension: hatch.dimension,
                    at: hatch.at,
                });
            }
            !is_mine
        });
        mine
    };
    if pending.is_empty() {
        return;
    }
    for hatch in pending {
        let owner = nearest_player(host, level, hatch.at);
        let mut nbt = Compound::new();
        nbt.insert(TAMED_TAG.into(), Tag::Byte(1));
        if let Some(uuid) = owner {
            // The API does not let a mod build a String tag (`JavaString` is not part of
            // its surface), so the owner is written in vanilla's own UUID form, the 4-int
            // array `NbtUtils.loadUUID` reads (`notes.md`, the API gap).
            nbt.insert(OWNER_TAG.into(), Tag::IntArray(uuid_ints(uuid)));
        }
        for (i, key) in HOME_TAGS.into_iter().enumerate() {
            nbt.insert(key.into(), Tag::Int(hatch.at[i].floor() as i32));
        }
        host.summon(dimension, DRAGON, hatch.at, nbt);
    }
}

/// A fire dragon is about to tick: the port replaces the template's AI with its own
/// flight and targeting, then cancels the vanilla tick.
pub(crate) fn dragon_tick(host: &mut dyn ServerHost, event: &mut EntityTickPreEvent<'_>) -> Flow {
    if event.kind != DRAGON {
        return Flow::Continue;
    }
    let Some(entity) = host.entity(event.entity) else {
        return Flow::Continue;
    };
    let Some(position) = entity.position(host) else {
        return Flow::Continue;
    };
    // A dead dragon is the game's, not the AI's. The port replaces the template's tick, so the
    // death tick that would remove it never runs here: left alone, a `/kill`ed dragon stays in
    // the world at 0 health and keeps flying (measured in game, slice 2). Its `die()` has
    // already rolled the drops by the time health reads 0, so the corpse is discarded here.
    if host.living(entity).is_some_and(|living| {
        living.alive(host) == Some(false) || living.health(host).is_some_and(|health| health <= 0.0)
    }) {
        entity.discard(host);
        return Flow::Continue;
    }
    // The template is a wandering trader; its own AI never runs here.
    let (tamed, home) = bond(host, entity, position);
    let players = if tamed {
        Vec::new()
    } else {
        entity
            .level(host)
            .map(|level| players_in(host, level))
            .unwrap_or_default()
    };
    let positions: Vec<[f64; 3]> = players.iter().map(|player| player.at).collect();
    let target = choose_target(position, &positions).map(|nearest| &players[nearest]);
    let motion = dragon_step(
        position,
        home,
        target.map(|player| player.at),
        tamed,
        DRAGON_FLIGHT_CEILING,
    );
    if motion.to != position {
        entity.teleport_to(host, motion.to, None);
    }
    bite(host, entity, position, target);
    Flow::Cancel
}

/// A player the dragon can see, as the tick needs it.
struct Nearby {
    uuid: u128,
    player: PlayerRef,
    at: [f64; 3],
}

/// One dragon's bite: the player it aims at and the animation's state. Upstream
/// keeps the target on the mob; a template mob has no field a port may write, so
/// the state lives here, keyed by the dragon's UUID (the `ARMING` pattern).
struct Attack {
    target: u128,
    bite: Bite,
}

/// Bites in progress, by dragon UUID. An animation is 35 ticks of presentation and
/// nothing saved depends on it: a world reload restarts it (recorded deviation).
static BITES: LazyLock<Mutex<HashMap<u128, Attack>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// What the melee state machine decided this tick, decided under the lock and acted
/// on after it is released (a host call inside the lock would be a hazard).
enum Melee {
    Land(PlayerRef),
    Forget,
    Idle,
}

/// The fire dragon's bite: upstream's `FireDragonEntity.aiStep` starts a bite when
/// the target is inside the dragon's inflated box, and `IafDragonLogic.updateDragonAttack`
/// lands it inside the window and the reach. A tamed dragon is not given a target
/// (`DragonAITargetNonTamedGoal.canUse` is false when tamed), so it never bites.
fn bite(host: &mut dyn ServerHost, dragon: EntityRef, position: [f64; 3], target: Option<&Nearby>) {
    let (Some(uuid), Some(dragon_box)) = (dragon.uuid(host), dragon.bounding_box(host)) else {
        return;
    };
    let plan = {
        let mut bites = BITES.lock().expect("iceandfire bite state lock poisoned");
        if bites.len() > 1024 {
            bites.clear();
        }
        let Some(target) = target else {
            // No target (tamed, or nobody in range): the bite in progress is dropped,
            // as upstream clears its target.
            bites.remove(&uuid);
            return;
        };
        let Some(attack) = bites.get_mut(&uuid) else {
            let reachable = target
                .player
                .entity()
                .bounding_box(host)
                .is_some_and(|player| intersects(grown(dragon_box, aggro_inflate()), player));
            if reachable {
                bites.insert(
                    uuid,
                    Attack {
                        target: target.uuid,
                        bite: Bite::start(),
                    },
                );
                return;
            }
            return;
        };
        if attack.target != target.uuid {
            // Upstream bites the target it has; a changed target restarts the animation.
            Melee::Forget
        } else {
            let step = attack.bite.step();
            let plan = if step.hit && within_reach(distance(position, target.at)) {
                Melee::Land(target.player)
            } else {
                Melee::Idle
            };
            if step.over {
                bites.remove(&uuid);
            }
            plan
        }
    };
    match plan {
        Melee::Land(player) => {
            // `IafDragonLogic.attackTarget`: `damageSources().mobAttack(this.dragon)`, so
            // the dragon is both the direct and the causing source.
            let _ = player.entity().hurt(
                host,
                "minecraft:mob_attack",
                AGE0_ATTACK_DAMAGE,
                Some(dragon),
                Some(dragon),
            );
        }
        Melee::Forget => {
            BITES
                .lock()
                .expect("iceandfire bite state lock poisoned")
                .remove(&uuid);
        }
        Melee::Idle => {}
    }
}

/// The dragon's tamed flag and home, from the tags the hatch wrote. A dragon
/// `/summon`ed by hand carries neither: it is wild and its home is where it is.
fn bond(host: &mut dyn ServerHost, entity: EntityRef, position: [f64; 3]) -> (bool, [f64; 3]) {
    let Some(data) = entity.nbt(host) else {
        return (false, position);
    };
    // `Tag::get` takes the key as a `&str` and looks it up without allocating a key; the
    // compound is a plain map keyed by Java strings, which a mod cannot name.
    let data = Tag::Compound(data);
    let tamed = matches!(data.get(TAMED_TAG), Some(Tag::Byte(1)));
    let mut home = position;
    for (i, key) in HOME_TAGS.into_iter().enumerate() {
        if let Some(Tag::Int(value)) = data.get(key) {
            home[i] = f64::from(*value);
        }
    }
    (tamed, home)
}

/// The living, non-spectator players of `level`.
fn players_in(host: &mut dyn ServerHost, level: LevelRef) -> Vec<Nearby> {
    let Some(players) = level.online_players(host) else {
        return Vec::new();
    };
    players
        .into_iter()
        .filter(|player| {
            let entity = player.entity();
            entity.spectator(host) != Some(true)
                && host
                    .living(entity)
                    .is_some_and(|living| living.alive(host) == Some(true))
        })
        .filter_map(|player| {
            let uuid = player.uuid();
            let at = player.entity().position(host)?;
            Some(Nearby { uuid, player, at })
        })
        .collect()
}

/// The UUID of the closest living, non-spectator player in `level` to `from`: the
/// hatch's owner. The egg bonds with the player nearest the *egg*
/// (`item.iceandfire.dragonegg.jei_desc`); the pre-migration port measured to the world
/// origin instead, which is fixed here.
fn nearest_player(host: &mut dyn ServerHost, level: LevelRef, from: [f64; 3]) -> Option<u128> {
    players_in(host, level)
        .into_iter()
        .min_by(|a, b| squared_distance(a.at, from).total_cmp(&squared_distance(b.at, from)))
        .map(|nearby| nearby.uuid)
}

/// `uuid` in vanilla's UUID NBT form: four ints, most significant half first
/// (`NbtUtils.createUUID`).
fn uuid_ints(uuid: u128) -> Vec<i32> {
    let hi = (uuid >> 64) as u64;
    let lo = uuid as u64;
    vec![(hi >> 32) as i32, hi as i32, (lo >> 32) as i32, lo as i32]
}

fn squared_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    dx * dx + dy * dy + dz * dz
}

/// The straight-line distance between two positions, as upstream's `distanceTo`.
fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    squared_distance(a, b).sqrt()
}

/// Whether the block at `pos` is a flame an egg incubates in.
fn flame_at(level: &dyn LevelHost, pos: [i32; 3]) -> bool {
    level
        .block_state(pos)
        .and_then(|state| level.state_block(state))
        .is_some_and(is_flame)
}

/// The `age` of an egg, from the state's serialized form
/// (`iceandfire:dragon_egg[age=12]`). The API hands a block part no property reader, and
/// the value is the only property this block has.
fn age_of(serialized: &str) -> u8 {
    serialized
        .split_once('[')
        .and_then(|(_, properties)| {
            properties
                .trim_end_matches(']')
                .split(',')
                .find_map(parse_age)
        })
        .unwrap_or(0)
}

fn parse_age(property: &str) -> Option<u8> {
    let (key, value) = property.split_once('=')?;
    if key.trim() != AGE_PROPERTY {
        return None;
    }
    value.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_age_is_read_from_the_serialized_state() {
        assert_eq!(age_of("iceandfire:dragon_egg[age=0]"), 0);
        assert_eq!(age_of("iceandfire:dragon_egg[age=71]"), 71);
        assert_eq!(age_of("iceandfire:dragon_egg[age=12,other=x]"), 12);
        assert_eq!(age_of("iceandfire:dragon_egg[other=x,age=7]"), 7);
    }

    #[test]
    fn a_state_without_an_age_is_age_zero() {
        assert_eq!(age_of("iceandfire:dragon_egg"), 0);
        assert_eq!(age_of("minecraft:stone"), 0);
        assert_eq!(age_of("iceandfire:dragon_egg[age=nonsense]"), 0);
    }

    #[test]
    fn an_owner_uuid_is_vanillas_four_ints() {
        assert_eq!(
            uuid_ints(0x1234_5678_9abc_def0_1122_3344_5566_7788),
            [0x1234_5678, 0x9abc_def0u32 as i32, 0x1122_3344, 0x5566_7788]
        );
        assert_eq!(uuid_ints(1), [0, 0, 0, 1]);
    }
}
