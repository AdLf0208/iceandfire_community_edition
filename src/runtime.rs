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
}
