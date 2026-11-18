//! Ice and Fire for Horizon: the fire dragon, end to end.
//!
//! The rules — the flame set, the egg's steps, the choice of target and the flight
//! — live in [`runtime`], which knows nothing about the game. This file is the one
//! `impl Hooks` the launcher subscribes, and it declares the two Parts whose
//! moments belong to one item and one block:
//!
//! * the red dragon egg **item** ([`RedDragonEgg`]): its placement is the moment the
//!   egg's incubation starts (`DragonEggEntity.java:134-160`);
//! * the dragon egg **block** ([`DragonEggBlock`]): its scheduled tick ages it one
//!   step per hundred ticks of flame and hatches it at the last age.
//!
//! The dragon itself is a mob of the carried content type (`iceandfire:fire_dragon`),
//! and a mob type keeps no per-tick Part at this API — a Part on a mob gets
//! `register_goals`/`register_behaviors`, whose contexts read the tick but cannot
//! move the mob. So its tick is the `entity_tick_pre` hook, scoped by `kind` to the
//! one type: see `notes.md` for the mapping and the measured reason.

pub mod glue;
pub mod runtime;

use mod_api::{
    self, BlockPart, EntityTickListenedEvent, EntityTickPreEvent, Flow, Hooks, ItemPart, Registrar,
    RegistrationContext, ServerHost,
};

/// The mod. It keeps no state of its own: the egg's chain lives in the level's
/// scheduled ticks and the hatch queue lives in [`glue`].
#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Attach the two Parts, once, at startup. The egg's block and the red egg item
    /// are the carried content's own ids, so a key that the content does not register
    /// is refused at load rather than silently never firing.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        let egg_block = r.block_key(glue::EGG_BLOCK);
        r.block_part(egg_block, DragonEggBlock);
        let egg_item = r.item_key(glue::EGG_ITEM);
        r.item_part(egg_item, RedDragonEgg);
    }

    /// The game tells no entity's tick unless a mod says it wants them: without this
    /// `entity_tick_pre` never fires (its own doc says so on `entity_tick_listened`).
    fn entity_tick_listened(
        &self,
        _host: &mut dyn ServerHost,
        event: &mut EntityTickListenedEvent,
    ) {
        event.listened = true;
    }

    /// A fire dragon is about to tick: fly it at its target, then cancel its
    /// template's tick so the wandering trader's AI, gravity and trading never run.
    /// The port is the dragon's AI (see `notes.md`).
    fn entity_tick_pre(
        &self,
        host: &mut dyn ServerHost,
        event: &mut EntityTickPreEvent<'_>,
    ) -> Flow {
        glue::dragon_tick(host, event)
    }

    /// A level finished its tick: arm the eggs placed this tick (the level's own tick
    /// is where a block tick may be asked for) and summon the dragons whose eggs
    /// hatched.
    fn level_tick_post(
        &self,
        host: mod_api::HostSide<'_>,
        event: &mut mod_api::LevelTickPostEvent,
    ) {
        let mod_api::HostSide::Server(host) = host else {
            return;
        };
        glue::level_tick(host, event.level);
    }
}

/// The red dragon egg item: placing it starts the egg block's incubation.
struct RedDragonEgg;

#[mod_api::horizon_part]
impl ItemPart for RedDragonEgg {
    /// `BlockItemExtension.onBlockPlacedBy`: the egg block is written; record it so the
    /// level's next tick starts the chain (a hook's server host cannot schedule a block
    /// tick itself, and the placement is not the level's own tick).
    fn on_block_placed_by(
        &self,
        host: &mut dyn ServerHost,
        event: &mut mod_api::OnBlockPlacedByEvent<'_>,
    ) {
        glue::egg_placed(host, event);
    }
}

/// The incubated egg block: one scheduled tick per `EGG_SCHEDULE_DELAY`, one `age`
/// step per hundred ticks of flame.
struct DragonEggBlock;

#[mod_api::horizon_part]
impl BlockPart for DragonEggBlock {
    fn tick_block(
        &self,
        level: &mut dyn mod_api::LevelWriteHost,
        event: &mut mod_api::TickBlockEvent,
    ) -> Flow {
        glue::egg_tick(level, event)
    }
}
