// Ice and Fire Community Edition — the fire dragon's egg and its flight (pure).
//
// Every number comes from the jar or its upstream source, cited at the constant.
// This module has no game types: the helper in `support.rs` reads the world and
// calls it, so the rules are testable without staging the game.

/// Ticks of flame a fire dragon's egg needs before it hatches
/// (`IafCommonConfig.java:45`, `eggBornTime` default 7200; used at
/// `DragonEggEntity.java:136`).
pub const EGG_BORN_TICKS: i32 = 7200;

/// Ticks of flame each `age` value of the port's egg block stands for. The
/// jar keeps a continuous counter on its egg entity; a block has no counter,
/// so progress lives in the block's own `age` state at this granularity.
/// `EGG_STEP_TICKS * (EGG_LAST_AGE + 1) == EGG_BORN_TICKS`.
pub const EGG_STEP_TICKS: i32 = 100;

/// The highest `age` value; an egg that reaches it in flame hatches.
pub const EGG_LAST_AGE: u8 = 71;

/// Ticks between two scheduled looks at an egg (the port's own cadence, the
/// block equivalent of the entity tick the jar gives its egg).
pub const EGG_SCHEDULE_DELAY: i32 = 100;

// The blockstate's granularity must add up to the jar's own incubation time.
const _: () = assert!(EGG_STEP_TICKS * (EGG_LAST_AGE as i32 + 1) == EGG_BORN_TICKS);

/// Where a wild dragon looks for a player: its `FOLLOW_RANGE` attribute
/// (`IafCommonConfig.java:68` `targetSearchLength` 128, set at
/// `DragonBaseEntity.java:295`).
pub const DRAGON_TARGET_RANGE: f64 = 128.0;

/// How far a dragon wanders from its home before turning back
/// (`wanderFromHomeDistance` 40, `IafCommonConfig.java:69`).
pub const DRAGON_HOME_LEASH: f64 = 40.0;

/// The dragon's flight ceiling (`maxFlight` 256, `IafCommonConfig.java:62`;
/// `isBeyondHeight` at `DragonBaseEntity.java:1443`).
pub const DRAGON_FLIGHT_CEILING: f64 = 256.0;

/// A hovering dragon's lift per tick (`IafDragonLogic.java:143`: `+0.08`
/// upward, `+0.12` in water).
pub const HOVER_RISE: f64 = 0.08;

/// A flying dragon's speed per tick at age 0, from the jar's flight formula
/// (`DragonBaseEntity.java:2090-2165`): `MOVEMENT_SPEED 0.15` × the air speed
/// modifier `5.2` × `0.1`, against the `0.9` per-tick damping of `travel` —
/// terminal speed `0.15 · 5.2 · 0.1 / 0.1 = 0.78` blocks/tick.
pub const FLIGHT_SPEED: f64 = 0.78;

/// The reach at which a flying dragon counts as tackling its target: the
/// dragon's box grown by 2 blocks must meet the target's
/// (`IafDragonLogic.java:112-118`).
pub const TACKLE_REACH: f64 = 2.0;

/// The fire dragon's bite animation length in ticks
/// (`FireDragonEntity.java:52`: `ANIMATION_BITE = Animation.create(35)`).
pub const BITE_ANIM_TICKS: i32 = 35;

/// The first tick of a bite that can land its hit (`IafDragonLogic.java:406`:
/// `getAnimationTick() > 15 && getAnimationTick() < 25`).
pub const BITE_HIT_FROM: i32 = 16;

/// The last tick of that window (`IafDragonLogic.java:406`).
pub const BITE_HIT_UNTIL: i32 = 24;

/// How far the dragon's own box is grown to notice a target it can bite
/// (`FireDragonEntity.java:107`: `inflate(2.5F + getRenderSize() * 0.33F)`).
pub const BITE_AGGRO_INFLATE: f64 = 2.5;

/// The render-size term of that growth (`FireDragonEntity.java:107`).
pub const BITE_AGGRO_SIZE_FACTOR: f64 = 0.33;

/// The reach of a landed bite, before its base
/// (`IafDragonLogic.java:405`: `getRenderSize() * 0.2574 * 2 + 2`).
pub const BITE_REACH_FACTOR: f64 = 0.2574;

/// The base reach of a landed bite (`IafDragonLogic.java:405`).
pub const BITE_REACH_BASE: f64 = 2.0;

/// A stage-1 dragon's render size at age 0: `DragonSize.GROWTH_STAGES[0].x0`
/// is 1 and the per-day step is multiplied by `ageFactor` 0
/// (`DragonSize.java:6-10`, `DragonBaseEntity.getRenderSize():1888`).
pub const AGE0_RENDER_SIZE: f64 = 1.0;

/// A dragon's `ATTACK_DAMAGE` at age 0: `FireDragonEntity.java:48` passes a
/// minimum of 1, `DragonBaseEntity.bakeAttributes` adds 1, and the 125-day
/// interpolation starts there.
pub const AGE0_ATTACK_DAMAGE: f32 = 1.0;

/// An axis-aligned box as `[[min x, y, z], [max x, y, z]]`, the shape the
/// engine's `getBoundingBox` reports.
pub type Box = [[f64; 3]; 2];

/// The dragon's box grown by the aggro inflation: the volume in which it
/// starts a bite.
#[must_use]
pub fn aggro_inflate() -> f64 {
    BITE_AGGRO_INFLATE + AGE0_RENDER_SIZE * BITE_AGGRO_SIZE_FACTOR
}

/// How far a landed bite reaches at age 0.
#[must_use]
pub fn bite_reach() -> f64 {
    AGE0_RENDER_SIZE * BITE_REACH_FACTOR * 2.0 + BITE_REACH_BASE
}

/// Whether a target `distance` blocks away is inside a bite's reach.
#[must_use]
pub fn within_reach(distance: f64) -> bool {
    distance < bite_reach()
}

/// `shape` grown by `by` blocks on every side, as `AABB.inflate` does.
#[must_use]
pub fn grown(shape: Box, by: f64) -> Box {
    let [min, max] = shape;
    [
        [min[0] - by, min[1] - by, min[2] - by],
        [max[0] + by, max[1] + by, max[2] + by],
    ]
}

/// Whether two boxes share any point (`AABB.intersects`).
#[must_use]
pub fn intersects(a: Box, b: Box) -> bool {
    (0..3).all(|i| a[0][i] < b[1][i] && b[0][i] < a[1][i])
}

/// What a bite has done at one tick of its animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiteTick {
    /// The bite lands its hit on this tick.
    pub hit: bool,
    /// The animation is over; the dragon may start another bite.
    pub over: bool,
}

/// One bite in progress: the animation's own tick and whether its hit has landed.
///
/// Upstream applies the hit on every tick of the window; the engine's hurt cooldown
/// absorbs all but the first, so this state applies it once. The animation counts its
/// own ticks rather than reading the entity's: `EntityRef::tick_count` reads **0** at
/// `entity_tick_pre` on this pin (measured in game, `notes.md`), so an animation keyed
/// on it would never reach its window. One `step` is one dragon tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bite {
    tick: i32,
    landed: bool,
}

impl Bite {
    /// A bite that starts now.
    #[must_use]
    pub fn start() -> Self {
        Bite {
            tick: 0,
            landed: false,
        }
    }

    /// Advance the animation one tick: whether the bite lands now and whether it is over.
    pub fn step(&mut self) -> BiteTick {
        self.tick += 1;
        let tick = bite_tick(self.tick);
        let hit = tick.hit && !self.landed;
        if hit {
            self.landed = true;
        }
        BiteTick {
            hit,
            over: tick.over,
        }
    }
}

/// What a bite does `elapsed` ticks after it started.
#[must_use]
pub fn bite_tick(elapsed: i32) -> BiteTick {
    BiteTick {
        hit: (BITE_HIT_FROM..=BITE_HIT_UNTIL).contains(&elapsed),
        over: elapsed >= BITE_ANIM_TICKS,
    }
}

/// Blocks an egg incubates in. Upstream asks its `uranus` library's
/// `BlockUtil.isBurning` (`DragonEggEntity.java:220`), which is not in the
/// mod's repository; the item's own description says "place in an open flame"
/// (`item.iceandfire.dragonegg.jei_desc`), so the port takes the blocks that
/// are an open flame or hotter.
pub const FLAME_BLOCKS: [&str; 6] = [
    "minecraft:fire",
    "minecraft:soul_fire",
    "minecraft:lava",
    "minecraft:magma_block",
    "minecraft:campfire",
    "minecraft:soul_campfire",
];

/// Whether `block` (a block id as the level holds it) is a flame an egg
/// incubates in.
pub fn is_flame(block: &str) -> bool {
    FLAME_BLOCKS.contains(&block)
}

/// One scheduled look at an egg.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EggStep {
    /// The `age` state value the egg should carry now.
    pub age: u8,
    /// Whether the dragon is due: the egg is replaced by it.
    pub hatch: bool,
    /// Whether the egg wants another scheduled tick (`false` once it hatched).
    pub again: bool,
}

/// Advances an egg by one [`EGG_STEP_TICKS`] look: in flame it ages one step,
/// and at [`EGG_LAST_AGE`] it hatches. Outside flame it keeps its age and asks
/// to be looked at again — upstream's egg never cools down
/// (`DragonEggEntity.java:219-225` increments only while burning).
pub fn egg_step(age: u8, burning: bool) -> EggStep {
    if !burning {
        return EggStep {
            age: age.min(EGG_LAST_AGE),
            hatch: false,
            again: true,
        };
    }
    if age >= EGG_LAST_AGE {
        EggStep {
            age: EGG_LAST_AGE,
            hatch: true,
            again: false,
        }
    } else {
        EggStep {
            age: age + 1,
            hatch: false,
            again: true,
        }
    }
}

/// What a dragon is doing this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flight {
    /// Rising in place (`IafDragonLogic.java:143`).
    Hover,
    /// Flying at its target (`DragonBaseEntity.java:2090-2165`).
    Pursue,
    /// At the target, inside tackle reach (`IafDragonLogic.java:112-118`).
    Hold,
    /// No target: flying back toward home (`wanderFromHomeDistance`).
    Return,
}

/// A dragon's next position and what it is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub to: [f64; 3],
    pub flight: Flight,
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn horizontal(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (dx, dz) = (a[0] - b[0], a[2] - b[2]);
    (dx * dx + dz * dz).sqrt()
}

/// The nearest player inside the dragon's search range
/// (`DragonAITargetNonTamedGoal`), as the index of `players`.
pub fn choose_target(pos: [f64; 3], players: &[[f64; 3]]) -> Option<usize> {
    players
        .iter()
        .enumerate()
        .map(|(i, p)| (i, distance(pos, *p)))
        .filter(|(_, d)| *d <= DRAGON_TARGET_RANGE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// One tick of a dragon's movement. `tamed` dragons never target a player
/// (`DragonAITargetNonTamedGoal.canUse` returns false when tamed); they hover,
/// or fly back to their home when more than the home leash from it. A dragon
/// never rises above `ceiling` (the level's height or `maxFlight`, whichever
/// is lower).
pub fn dragon_step(
    pos: [f64; 3],
    home: [f64; 3],
    target: Option<[f64; 3]>,
    tamed: bool,
    ceiling: f64,
) -> Motion {
    let ceiling = ceiling.min(DRAGON_FLIGHT_CEILING);
    // A tamed dragon is not a threat to players (upstream's tamed target goal
    // only hunts food animals, which this slice does not implement).
    let target = if tamed { None } else { target };
    if let Some(target) = target {
        let d = distance(pos, target);
        if d <= TACKLE_REACH {
            return Motion {
                to: pos,
                flight: Flight::Hold,
            };
        }
        let step = FLIGHT_SPEED.min(d);
        let scale = step / d;
        let mut to = [
            pos[0] + (target[0] - pos[0]) * scale,
            pos[1] + (target[1] - pos[1]) * scale,
            pos[2] + (target[2] - pos[2]) * scale,
        ];
        to[1] = to[1].min(ceiling);
        return Motion {
            to,
            flight: Flight::Pursue,
        };
    }
    // No target: home is a leash when the dragon has strayed, else hover.
    if tamed && horizontal(pos, home) > DRAGON_HOME_LEASH {
        let d = horizontal(pos, home);
        let step = FLIGHT_SPEED.min(d);
        let scale = step / d;
        let mut to = [
            pos[0] + (home[0] - pos[0]) * scale,
            pos[1] + (home[1] - pos[1]).clamp(-FLIGHT_SPEED, FLIGHT_SPEED),
            pos[2] + (home[2] - pos[2]) * scale,
        ];
        to[1] = to[1].min(ceiling);
        return Motion {
            to,
            flight: Flight::Return,
        };
    }
    let mut to = pos;
    if pos[1] < ceiling {
        to[1] = (pos[1] + HOVER_RISE).min(ceiling);
    }
    Motion {
        to,
        flight: Flight::Hover,
    }
}

/// `uuid` (the pin's `u128` form of a player's UUID) as the hyphenated string
/// the egg's `OwnerUUID` tag holds (`DragonEggEntity.java:71`: `EntityReference`).
pub fn format_uuid(uuid: u128) -> String {
    let hex = format!("{uuid:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < 1e-9)
    }

    #[test]
    fn flame_blocks_incubate_and_others_do_not() {
        for block in [
            "minecraft:fire",
            "minecraft:soul_fire",
            "minecraft:lava",
            "minecraft:magma_block",
            "minecraft:campfire",
            "minecraft:soul_campfire",
        ] {
            assert!(is_flame(block), "{block} is an open flame");
        }
        for block in [
            "minecraft:stone",
            "minecraft:water",
            "minecraft:air",
            "minecraft:dirt",
        ] {
            assert!(!is_flame(block), "{block} is not a flame");
        }
    }

    #[test]
    fn an_egg_outside_flame_keeps_its_age_and_waits() {
        let step = egg_step(5, false);
        assert_eq!(
            step,
            EggStep {
                age: 5,
                hatch: false,
                again: true
            }
        );
    }

    #[test]
    fn an_egg_in_flame_ages_one_step() {
        let step = egg_step(0, true);
        assert_eq!(
            step,
            EggStep {
                age: 1,
                hatch: false,
                again: true
            }
        );
    }

    #[test]
    fn the_last_step_hatches_and_stops_the_chain() {
        let step = egg_step(EGG_LAST_AGE, true);
        assert_eq!(
            step,
            EggStep {
                age: EGG_LAST_AGE,
                hatch: true,
                again: false
            }
        );
    }

    #[test]
    fn a_full_incubation_is_the_jars_7200_ticks() {
        let mut age = 0;
        let mut ticks = 0;
        loop {
            let step = egg_step(age, true);
            ticks += EGG_STEP_TICKS;
            age = step.age;
            if step.hatch {
                break;
            }
            assert!(step.again, "an egg that has not hatched keeps ticking");
        }
        assert_eq!(
            ticks, EGG_BORN_TICKS,
            "100-tick steps of 72 ages are 7200 ticks"
        );
        assert_eq!(usize::from(EGG_LAST_AGE) + 1, 72);
    }

    #[test]
    fn a_tamed_dragon_hovers_upward() {
        let motion = dragon_step([0.5, 64.0, 0.5], [0.5, 64.0, 0.5], None, true, 320.0);
        assert_eq!(motion.flight, Flight::Hover);
        assert!(
            close(motion.to, [0.5, 64.0 + HOVER_RISE, 0.5]),
            "{:?}",
            motion.to
        );
    }

    #[test]
    fn a_dragon_never_rises_through_the_ceiling() {
        let motion = dragon_step([0.5, 255.99, 0.5], [0.5, 255.99, 0.5], None, true, 320.0);
        assert!(motion.to[1] <= DRAGON_FLIGHT_CEILING, "{:?}", motion.to);
        let at_ceiling = dragon_step(
            [0.5, DRAGON_FLIGHT_CEILING, 0.5],
            [0.5, 0.0, 0.5],
            None,
            true,
            320.0,
        );
        assert!(
            close(at_ceiling.to, [0.5, DRAGON_FLIGHT_CEILING, 0.5]),
            "{:?}",
            at_ceiling.to
        );
    }

    #[test]
    fn a_wild_dragon_flying_at_a_player_closes_by_one_tick_of_flight() {
        let pos = [0.0, 70.0, 0.0];
        let target = [20.0, 70.0, 0.0];
        let motion = dragon_step(pos, pos, Some(target), false, 320.0);
        assert_eq!(motion.flight, Flight::Pursue);
        assert!(
            close(motion.to, [FLIGHT_SPEED, 70.0, 0.0]),
            "{:?}",
            motion.to
        );
        assert!(distance(motion.to, target) < distance(pos, target));
    }

    #[test]
    fn a_pursuing_dragon_climbs_toward_a_target_above_it() {
        let motion = dragon_step(
            [0.0, 70.0, 0.0],
            [0.0, 70.0, 0.0],
            Some([0.0, 80.0, 0.0]),
            false,
            320.0,
        );
        assert!(motion.to[1] > 70.0, "{:?}", motion.to);
    }

    #[test]
    fn a_dragon_inside_tackle_reach_holds_still() {
        let pos = [0.0, 70.0, 0.0];
        let target = [1.0, 70.0, 0.0];
        let motion = dragon_step(pos, pos, Some(target), false, 320.0);
        assert_eq!(motion.flight, Flight::Hold);
        assert!(close(motion.to, pos), "{:?}", motion.to);
    }

    #[test]
    fn a_tamed_dragon_ignores_a_player_it_could_hunt() {
        let pos = [0.0, 70.0, 0.0];
        let motion = dragon_step(pos, pos, Some([1.0, 70.0, 0.0]), true, 320.0);
        assert_ne!(motion.flight, Flight::Hold);
        assert_ne!(motion.flight, Flight::Pursue);
    }

    #[test]
    fn a_tamed_dragon_beyond_its_leash_flies_home() {
        let home = [0.0, 64.0, 0.0];
        let pos = [DRAGON_HOME_LEASH + 10.0, 80.0, 0.0];
        let motion = dragon_step(pos, home, None, true, 320.0);
        assert_eq!(motion.flight, Flight::Return);
        assert!(horizontal(motion.to, home) < horizontal(pos, home));
    }

    #[test]
    fn the_nearest_player_inside_the_search_range_is_the_target() {
        let pos = [0.0, 64.0, 0.0];
        let players = [[10.0, 64.0, 0.0], [3.0, 64.0, 0.0]];
        assert_eq!(choose_target(pos, &players), Some(1));
    }

    #[test]
    fn a_player_beyond_the_search_range_is_not_a_target() {
        let pos = [0.0, 64.0, 0.0];
        let players = [[DRAGON_TARGET_RANGE + 1.0, 64.0, 0.0]];
        assert_eq!(choose_target(pos, &players), None);
    }

    #[test]
    fn an_owner_uuid_is_the_hyphenated_form() {
        assert_eq!(format_uuid(1), "00000000-0000-0000-0000-000000000001");
        assert_eq!(
            format_uuid(0x1234_5678_9abc_def0_1122_3344_5566_7788),
            "12345678-9abc-def0-1122-334455667788"
        );
    }

    // --- the bite (slice 2) -------------------------------------------------------

    #[test]
    fn a_bite_animation_is_the_jars_35_ticks() {
        assert!(
            !bite_tick(34).over,
            "34 ticks is still inside the animation"
        );
        assert!(bite_tick(35).over, "35 ticks ends it (FireDragonEntity:52)");
    }

    #[test]
    fn a_bite_lands_only_inside_its_window() {
        assert!(!bite_tick(15).hit, "tick 15 is before the window");
        assert!(bite_tick(16).hit, "tick 16 is the window's first tick");
        assert!(bite_tick(24).hit, "tick 24 is the window's last tick");
        assert!(!bite_tick(25).hit, "tick 25 is past the window");
    }

    #[test]
    fn a_bite_lands_once_however_the_window_is_walked() {
        let mut bite = Bite::start();
        let mut hits = Vec::new();
        let mut over_at = None;
        for tick in 1..=BITE_ANIM_TICKS {
            let step = bite.step();
            if step.hit {
                hits.push(tick);
            }
            if step.over {
                over_at = Some(tick);
            }
        }
        assert_eq!(
            hits,
            vec![BITE_HIT_FROM],
            "one landed bite, on the window's first tick (the engine's hurt cooldown absorbs the rest)"
        );
        assert_eq!(
            over_at,
            Some(BITE_ANIM_TICKS),
            "the animation ends on its 35th tick"
        );
    }

    #[test]
    fn a_dragon_lands_one_bite_per_animation_and_starts_the_next() {
        let mut bite = Bite::start();
        let mut hits = 0;
        for _ in 1..=(2 * BITE_ANIM_TICKS) {
            let step = bite.step();
            if step.hit {
                hits += 1;
            }
            if step.over {
                bite = Bite::start();
            }
        }
        assert_eq!(hits, 2, "one landed bite per 35-tick animation");
    }

    #[test]
    fn the_aggro_box_at_age_zero_is_the_jars_two_point_eight_three_blocks() {
        let inflate = aggro_inflate();
        assert!(
            (inflate - 2.83).abs() < 1e-9,
            "2.5 + renderSize 1.0 * 0.33 = 2.83, got {inflate}"
        );
    }

    #[test]
    fn a_target_just_inside_the_aggro_box_is_bitten_and_one_outside_is_not() {
        let dragon = [[0.0, 0.0, 0.0], [0.6, 1.95, 0.6]];
        // A 0.6-wide player box whose near face is 2.7 blocks away along x.
        let near = [[3.3, 0.0, 0.0], [3.9, 1.8, 0.6]];
        assert!(
            intersects(grown(dragon, aggro_inflate()), near),
            "a 2.7-block gap is inside the 2.83-block aggro box"
        );
        let far = [[3.5, 0.0, 0.0], [4.1, 1.8, 0.6]];
        assert!(
            !intersects(grown(dragon, aggro_inflate()), far),
            "a 2.9-block gap is outside it"
        );
    }

    #[test]
    fn a_bite_reaches_the_jars_two_and_a_half_blocks() {
        let reach = bite_reach();
        assert!(
            (reach - 2.5148).abs() < 1e-9,
            "renderSize 1.0 * 0.2574 * 2 + 2 = 2.5148, got {reach}"
        );
        assert!(within_reach(2.5), "2.5 blocks is inside the reach");
        assert!(!within_reach(2.6), "2.6 blocks is outside it");
    }

    #[test]
    fn at_age_zero_a_bite_takes_the_jars_attack_damage() {
        assert!(
            (AGE0_ATTACK_DAMAGE - 1.0).abs() < f32::EPSILON,
            "FireDragonEntity's ATTACK_DAMAGE attribute is 1 at age 0"
        );
    }
}
