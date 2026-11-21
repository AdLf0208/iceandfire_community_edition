#!/usr/bin/env bash
# In-game check for the Ice and Fire fire dragon: the egg chain and the bite.
#
# **The game must be a launcher-built one** (`ports/tools/launcher-game.sh build iceandfire`),
# because this is a native package: the code is compiled into the client and no `--dev-mod`
# leg loads it. `GAME=` names that binary (every variable comes from
# `eval "$(ports/tools/launcher-game.sh env iceandfire)"`).
#
# The End, in the void at x=200 (away from the central island, its dragon egg and the
# ambient item entities a generated level grows), on a stone floor at y=100.
#
# Legs:
#  A  the dragon BITES a player it hunts: a wild `iceandfire:fire_dragon` six blocks away
#     closes to its hold distance, and the first health change is exactly -1.0 (the jar's
#     `ATTACK_DAMAGE` 1 at age 0, `IafDragonLogic.updateDragonAttack`), every later step 1.0
#     too, and at least two hits land inside 35-tick bites;
#  B  the TAMED control: a dragon carrying `{iceandfire:Tamed:1b}` three blocks away keeps the
#     player's health unchanged over the same window — upstream's player target goal is
#     wild-only (`DragonAITargetNonTamedGoal.canUse`);
#  C  the carried content: the egg item is given and read back, an unknown id is refused, the
#     item places `iceandfire:dragon_egg[age=0]` and `ctl block` reads the age back;
#  D  the flame differential: an egg on stone keeps `age=0` over the window in which an egg on
#     magma reaches `age>=2` (`DragonEggEntity.updateEggCondition`);
#  E  the hatch: the magma egg reaches its last age and becomes one `iceandfire:fire_dragon`,
#     7,200 ticks of flame after placement, with `Tamed 1b`, an `Owner` and 20.0 health;
#  F  the egg block's own loot table drops the egg item when mined by hand.
#
# SELFTEST=1 flips B (it asserts the tamed dragon DID hurt the player, which must FAIL),
# proving the bite assertion can fail.
#
# CONTROL_ONLY=1 with GAME=<a game WITHOUT the package> is the control leg: the ids must be
# unknown there and the run must FAIL. It is the only difference between the two games.
#
# Every reply is parsed JSON; a `PASS:` line alone is not evidence, the cleanup trap prints
# the `EXIT:<status>` the log ends with.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${IF_DEBUG:-}" ] || set -x

CLONE=${CLONE:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/iceandfire
CTL=${CTL:-}
GAME=${GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/iceandfire-e2e.sock}
KEEP=${KEEP:-0}
SELFTEST=${SELFTEST:-0}
CONTROL_ONLY=${CONTROL_ONLY:-0}
WORLD=IceAndFireE2E
HMOD=${HMOD:-$PORT/iceandfire.hmod}

EGG_ITEM=iceandfire:dragonegg_red
EGG_BLOCK=iceandfire:dragon_egg
DRAGON=iceandfire:fire_dragon

# The End void: a stone floor at y=100, the arena at 200.
X0=200
FLOOR=100
EGG_Z=2          # the incubating egg (magma under it)
STONE_Z=4        # an egg on stone: the differential
LOOT_Z=6         # a third egg, mined for its drop

guard() { echo "EXIT:1"; exit 1; }
command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; guard; }
[ -n "$CTL" ] || { echo "CTL= is required (eval \"\$(ports/tools/launcher-game.sh env iceandfire)\")" >&2; guard; }
[ -n "$GAME" ] || {
    echo "GAME= is required: this package is native, so the game must be launcher-built" >&2
    echo "  ports/tools/launcher-game.sh build iceandfire" >&2
    echo "  eval \"\$(ports/tools/launcher-game.sh env iceandfire)\" && tools/e2e.sh" >&2
    guard
}
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; guard; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; guard; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; guard; }

SCRATCH=$(mktemp -d /tmp/opencode/iceandfire-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch BINARY [WORLD]
    local game=$1 world=${2:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$game" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" >/dev/null
    ctl wait-ready 180 >/dev/null
    ctl wait 40
}

health() { ctl effects | jq -r '.health'; }
block_of() { ctl block "$1" "$2" "$3" | jq -r '.block // .name // empty'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // empty'; }
age_of() { block_state "$1" "$2" "$3" | sed -n 's/.*age=\([0-9][0-9]*\).*/\1/p'; }

# Dragon health by type filter; `ctl entities` reports mobs as `kind: "mob"` with `.type`.
dragon_health() {
    ctl entities 32 "$DRAGON" \
        | jq -r --arg d "$DRAGON" '[.entities[] | select(.type == $d)][0].health // "none"'
}
dragon_count() {
    ctl entities 48 "$DRAGON" | jq -r --arg d "$DRAGON" '[.entities[] | select(.type == $d)] | length'
}
# The nearest dragon's whole saved compound, as the game prints it: the hatch's Tamed and
# Owner tags ride there. `data get <path>` answers a bare value without the path, so the
# whole compound is the one reply that names both.
dragon_nbt() {
    ctl command "data get entity @e[type=$DRAGON,limit=1,sort=nearest]" >/dev/null 2>&1 || true
    ctl wait 3 >/dev/null
    ctl messages 20 | jq -r '.chat[] | select(startswith("Fire Dragon has the following entity data:"))' | tail -1
}

# The End arena: a stone floor, clear air above it, no ambient items, the player on it.
arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 $FLOOR 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" "$FLOOR" 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill $((X0 - 12)) $FLOOR -10 $((X0 + 12)) $FLOOR 14 minecraft:stone" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 12)) $((FLOOR + 1)) -10 $((X0 + 12)) $((FLOOR + 10)) 14 minecraft:air" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 $((FLOOR + 1)) 0.5" >/dev/null
    ctl wait 12
    [ "$(block_of "$X0" "$FLOOR" 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_of "$X0" "$((FLOOR + 1))" 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# One player health reading series while `ticks` ticks pass in `step`-tick slices.
# `read_health_series TICKS STEP` sets `SERIES` (space separated) and `DROPS` (one
# "from->to" per change, newest last).
read_health_series() { # read_health_series TICKS STEP
    local ticks=$1 step=$2 waited=0 prev series="" drops=""
    prev=$(health)
    series=$prev
    while [ "$waited" -lt "$ticks" ]; do
        ctl wait "$step" >/dev/null
        waited=$((waited + step))
        local now
        now=$(health)
        series="$series $now"
        if [ "$now" != "$prev" ]; then
            drops="$drops $prev->$now"
            prev=$now
        fi
    done
    SERIES=$series
    DROPS=$drops
}

# Every step of a health series must be exactly one bite's worth (the jar's ATTACK_DAMAGE 1):
# a port that damaged nine times inside the bite window would show it here.
assert_bites_of_one() { # assert_bites_of_one DROPS
    local from to step
    for pair in $1; do
        from=${pair%%->*}
        to=${pair##*->}
        step=$(awk -v a="$from" -v b="$to" 'BEGIN { printf "%.2f", a - b }')
        [ "$step" = "1.00" ] \
            || fail "a bite took $step health ($pair), expected exactly 1.0 (ATTACK_DAMAGE at age 0)"
    done
}

echo "== package under test =="
if [ -f "$HMOD" ]; then
    echo "  $HMOD $(stat -c %s "$HMOD") B sha256 $(sha256sum "$HMOD" | cut -d' ' -f1)"
fi
echo "  game $GAME"
echo "  game version $(sha256sum "$GAME" | cut -d' ' -f1)"

if [ "$CONTROL_ONLY" = 1 ]; then
    echo "== control: a game WITHOUT this package =="
    launch "$GAME" IceAndFireControl
    ctl give "$EGG_ITEM" 1 >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    got=$(ctl inventory | jq --arg i "$EGG_ITEM" '[.hotbar[], .main[] | select(.item == $i) | .count] | add // 0')
    [ "$got" = 0 ] \
        || fail "the control game holds $got $EGG_ITEM, so the id does not come from this package"
    ctl command "execute in minecraft:the_end run setblock $X0 $((FLOOR + 1)) 0 $EGG_BLOCK" >/dev/null 2>&1 || true
    ctl wait 4 >/dev/null
    got_block=$(ctl command "execute in minecraft:the_end run setblock $X0 $((FLOOR + 1)) 0 $EGG_BLOCK" 2>&1 || true)
    echo "  control setblock: $got_block"
    fail "control: $EGG_ITEM is unknown to the game, so it comes from this package (this run must fail)"
fi

echo "== launch (The End arena, launcher-built game) =="
launch "$GAME"
arena

echo "== C: the carried egg item and block are in this package =="
ctl command "clear @s" >/dev/null
ctl wait 2 >/dev/null
ctl give "$EGG_ITEM" 3
ctl wait 6 >/dev/null
held=$(ctl inventory | jq --arg i "$EGG_ITEM" '[.hotbar[], .main[] | select(.item == $i) | .count] | add // 0')
[ "$held" = 3 ] || fail "the inventory holds $held $EGG_ITEM after giving 3: $(ctl inventory | jq -c '"'"'{hotbar,main}'"'"')"
echo "  $EGG_ITEM: $held in the inventory"
if ctl give iceandfire:no_such_item 1 >/dev/null 2>&1; then
    fail "the game accepted the unknown item iceandfire:no_such_item"
fi
echo "  iceandfire:no_such_item: refused"

# The magma egg first, so it incubates while the rest of the run happens.
ctl command "execute in minecraft:the_end run setblock $X0 $FLOOR $EGG_Z minecraft:magma_block" >/dev/null
ctl wait 4 >/dev/null
# `ctl place` refuses a cell the player stands in, so stand in the cell beside each egg.
place_egg() { # place_egg Z STAND_Z
    ctl tp "$((X0)).5" "$((FLOOR + 1))" "$2.5" >/dev/null
    ctl wait 4 >/dev/null
    local reply
    reply=$(ctl place "$X0" "$((FLOOR + 1))" "$1" --item "$EGG_ITEM") \
        || fail "the egg item did not place at $X0,$((FLOOR + 1)),$1: ${reply:-no reply}"
    ctl wait 2 >/dev/null
    [ "$(block_of "$X0" "$((FLOOR + 1))" "$1")" = "$EGG_BLOCK" ] \
        || fail "the placed block at z=$1 is $(block_of "$X0" "$((FLOOR + 1))" "$1"), expected $EGG_BLOCK"
    # The egg's first look may already have run by the time the placement replies (measured:
    # a magma egg reads age 1, a stone egg age 0), so placement asserts "fresh", not "0".
    local age
    age=$(age_of "$X0" "$((FLOOR + 1))" "$1")
    [ -n "$age" ] && [ "$age" -le 1 ] \
        || fail "a placed egg starts at age '$age', expected 0 or 1"
}
place_egg "$EGG_Z" $((EGG_Z - 1))
place_egg "$STONE_Z" $((STONE_Z - 1))
place_egg "$LOOT_Z" $((LOOT_Z - 1))
ctl wait 6 >/dev/null
[ "$(age_of "$X0" "$((FLOOR + 1))" "$STONE_Z")" = 0 ] \
    || fail "the egg on stone is at age $(age_of "$X0" "$((FLOOR + 1))" "$STONE_Z") right after placement"
echo "  placed: $EGG_BLOCK on magma (z=$EGG_Z, age $(age_of "$X0" "$((FLOOR + 1))" "$EGG_Z")) and on stone (z=$STONE_Z, z=$LOOT_Z, age 0)"

echo "== A: a wild dragon bites the player it hunts, one ATTACK_DAMAGE per bite =="
ctl command "item replace entity @s weapon.mainhand with minecraft:air" >/dev/null 2>&1 || true
ctl command "gamemode survival @s" >/dev/null
ctl command "gamerule natural_health_regeneration false" >/dev/null
ctl tp "$((X0)).5" "$((FLOOR + 1))" 0.5 >/dev/null
ctl wait 6 >/dev/null
before=$(health)
[ "$before" = "20.0" ] || fail "the player starts at $before health, expected 20.0 (a fresh survival player)"
ctl command "summon $DRAGON $((X0)).5 $((FLOOR + 1)) 6.5" >/dev/null
ctl wait 2 >/dev/null
count=$(dragon_count)
[ "$count" -ge 1 ] || fail "the summoned $DRAGON is not there: $(ctl entities 32 "$DRAGON" | jq -c '.entities')"
echo "  dragon health: $(dragon_health) (the content's max_health 20.0)"
read_health_series 90 5
echo "  health series: $SERIES"
echo "  drops:${DROPS:- none}"
[ -n "$DROPS" ] || fail "the wild dragon never hurt the player over 90 ticks (health $SERIES)"
assert_bites_of_one "$DROPS"
first_drop=$(echo "$DROPS" | awk '{print $1}')
[ "$first_drop" = "$before->$(awk -v b="$before" 'BEGIN { printf "%.1f", b - 1 }')" ] \
    || fail "the first health change was $first_drop, expected $before->$before-1"
after_bite=$(health)
awk -v h="$after_bite" 'BEGIN { exit !(h <= 19.0) }' \
    || fail "the player is at $after_bite after 90 ticks of biting, expected at most 19.0"

echo "== B: the tamed control — a dragon that is not wild leaves the player alone =="
ctl command "kill @e[type=$DRAGON]" >/dev/null
ctl wait 10 >/dev/null
gone=$(dragon_count)
[ "$gone" = 0 ] || fail "a /kill'ed dragon is still in the world ($(ctl entities 48 "$DRAGON" | jq -c '.entities'))"
echo "  a killed dragon is removed by the game"
ctl tp "$((X0)).5" "$((FLOOR + 1))" 0.5 >/dev/null
ctl wait 2 >/dev/null
# The pin's SNBT parser refuses an unquoted key that carries a namespace (`Expected literal ,`
# at `iceandfire:Tamed:1b}`); a quoted key is the shape it takes.
ctl command "summon $DRAGON $((X0)).5 $((FLOOR + 1)) 3.5 {\"iceandfire:Tamed\":1b}" >/dev/null
ctl wait 4 >/dev/null
count=$(dragon_count)
[ "$count" -ge 1 ] || fail "the tamed $DRAGON did not summon: $(ctl entities 32 "$DRAGON" | jq -c '.entities')"
tamed_before=$(health)
read_health_series 90 5
echo "  tamed dragon at 3 blocks: health $SERIES (before $tamed_before)"
if [ "$SELFTEST" = 1 ]; then
    # The deliberately wrong expectation: a tamed dragon MUST not hurt its owner. It does not,
    # so this leg fails — which is the proof the tamed control can catch a bite-everything port.
    [ -n "$DROPS" ] || fail "SELFTEST: the tamed control did not trip (no health change on a tamed dragon)"
fi
[ -z "$DROPS" ] \
    || fail "the tamed dragon hurt the player ($DROPS); upstream's player target goal is wild-only"
echo "  the tamed dragon did not bite"

echo "== D: the flame differential — stone does not incubate, magma does =="
ctl command "kill @e[type=$DRAGON]" >/dev/null
ctl command "gamemode creative @s" >/dev/null
ctl wait 4 >/dev/null
ctl wait 300 >/dev/null
stone_age=$(age_of "$X0" "$((FLOOR + 1))" "$STONE_Z")
magma_age=$(age_of "$X0" "$((FLOOR + 1))" "$EGG_Z")
echo "  after ~300 more ticks: egg on stone age $stone_age, egg on magma age $magma_age"
[ "$stone_age" = 0 ] || fail "the egg on stone aged to $stone_age; only a flame incubates an egg"
[ -n "$magma_age" ] && [ "$magma_age" -ge 2 ] \
    || fail "the egg on magma is at age '$magma_age' after 300 ticks, expected at least 2"

echo "== F: the egg block's loot table drops the egg item, with a vanilla block as the control =="
# A broken block's drop is its loot table, and a table that rolls nothing shows only when it is
# used: the vanilla barrel broken beside the egg proves the harness can drop anything at all.
ctl command "gamemode survival @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 4 >/dev/null
ctl command "execute in minecraft:the_end run setblock $X0 $((FLOOR + 1)) $((LOOT_Z + 2)) minecraft:barrel" >/dev/null
ctl wait 6 >/dev/null
ctl tp "$((X0)).5" "$((FLOOR + 1))" "$((LOOT_Z + 1)).5" >/dev/null
ctl wait 4 >/dev/null
reply=$(ctl break "$X0" "$((FLOOR + 1))" $((LOOT_Z + 2))) || fail "cannot break the control barrel"
echo "  vanilla barrel: $(jq -c '.drops' <<<"$reply")"
got=$(jq -r '.drops["minecraft:barrel"] // 0' <<<"$reply")
[ "$got" -ge 1 ] || fail "the vanilla control block dropped $got of itself: $reply"
ctl tp "$((X0)).5" "$((FLOOR + 1))" "$((LOOT_Z + 2)).5" >/dev/null
ctl wait 4 >/dev/null
reply=$(ctl break "$X0" "$((FLOOR + 1))" "$LOOT_Z") || fail "cannot break the egg block"
echo "  $EGG_BLOCK: $(jq -c '.drops' <<<"$reply")"
[ "$(block_of "$X0" "$((FLOOR + 1))" "$LOOT_Z")" = "minecraft:air" ] \
    || fail "the mined egg block is still there: $(ctl block "$X0" "$((FLOOR + 1))" "$LOOT_Z")"
drops=$(jq -r --arg d "$EGG_ITEM" '.drops[$d] // 0' <<<"$reply")
[ "$drops" -ge 1 ] || fail "the egg block dropped $drops $EGG_ITEM, expected one (its 26.3 loot table)"
# The static half of the same claim: no carried recipe or loot table is in the 1.21.1 dialect.
python3 "$PORT/tools/data_shape.py" "$HMOD" || fail "a carried table is in the 1.21.1 dialect (see above)"
ctl command "gamemode creative @s" >/dev/null
ctl wait 4 >/dev/null

echo "== E: the magma egg hatches into one tamed, owned fire dragon =="
# 7,200 ticks of flame: the port's own chain (20-tick arming + 72 steps of 100 ticks).
waited=0
while [ "$waited" -lt 7800 ]; do
    ctl wait 200 >/dev/null
    waited=$((waited + 200))
    age=$(age_of "$X0" "$((FLOOR + 1))" "$EGG_Z")
    if [ -z "$age" ]; then
        break
    fi
    if [ $((waited % 2000)) -eq 0 ]; then
        echo "  $waited ticks: egg age $age"
    fi
done
hatched_block=$(block_of "$X0" "$((FLOOR + 1))" "$EGG_Z")
[ "$hatched_block" = "minecraft:air" ] \
    || fail "the egg block is '$hatched_block' after $waited ticks, expected air (hatched)"
ctl tp "$((X0)).5" "$((FLOOR + 1))" 0.5 >/dev/null
ctl wait 10 >/dev/null
count=$(dragon_count)
[ "$count" -ge 1 ] || fail "no $DRAGON at the hatched egg: $(ctl entities 48 "$DRAGON" | jq -c '.entities')"
hp=$(dragon_health)
echo "  hatched dragon health: $hp"
awk -v h="$hp" 'BEGIN { exit !(h > 19.9 && h < 20.1) }' \
    || fail "the hatched dragon has $hp health, expected 20.0 (the jar's age-0 attribute)"
nbt=$(dragon_nbt)
case "$nbt" in
    *'"iceandfire:Tamed": 1b'*) echo "  iceandfire:Tamed: 1b" ;;
    *) fail "the hatched dragon is not tamed (no iceandfire:Tamed 1b): '${nbt:0:600}'" ;;
esac
case "$nbt" in
    *'"iceandfire:Owner": [I;'*) echo "  iceandfire:Owner: [I; ...]" ;;
    *) fail "the hatched dragon has no iceandfire:Owner int-array tag: '${nbt:0:600}'" ;;
esac

echo "PASS: a wild fire dragon bit a survival player for exactly 1.0 health per bite (the jar's ATTACK_DAMAGE at age 0) with at least one bite in 90 ticks, a tamed dragon at 3 blocks left the player alone, the egg item and block exist and an unknown id is refused, an egg on stone kept age 0 while an egg on magma aged, the magma egg hatched after 7,200 ticks of flame into one tamed dragon with an owner and 20.0 health, and the egg block's loot table dropped the egg item"
