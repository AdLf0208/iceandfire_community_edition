#!/usr/bin/env python3
"""Check that the data a package carries is in 26.3's dialect, not 1.21.1's.

The same idea as `ports/modularrouters/tools/data_shape.py`, generalised to the
errors this campaign has proved in game (each of them passes `cargo horizon-mod
check`, `fits` and every unit test, and shows only when the data is used):

* a **recipe ingredient** written as `{"item": "..."}` / `{"tag": "..."}`. 26.3
  takes a bare string (`"minecraft:iron_ingot"`, `"#minecraft:arrows"`) in a
  shaped recipe's `key` and a shapeless recipe's `ingredients`; the object form
  is rejected at load and the recipe never crafts.
* a **loot-table pool condition** written as `"conditions": [ {...} ]`. 26.3's
  own tables (all 1139 block tables in the prepared 26.3 resources) express it
  as a singular `"condition": {...}` whose object names its kind with `"type"`
  (`{"type": "minecraft:survives_explosion"}`); with the 1.21.1 array form the
  block **drops nothing at all** (`modularrouters` measured 0 dropped while a
  vanilla table beside it dropped 1).
* the 1.21.1 **`bonus_rolls`** key on a loot pool, which the 26.3 shape has no
  place for (and 26.3's pools spell an integral `rolls` as `1`, not `1.0`).

    python3 tools/data_shape.py <package.hmod> [more.hmod]
    python3 tools/data_shape.py --data <dir> [more dirs/files]

Exit 0 when every carried recipe and loot table is in the 26.3 dialect, 1
otherwise, printing each offender with the value it found.
"""
from __future__ import annotations

import json
import sys
import zipfile
from pathlib import Path

CRAFTING = ("minecraft:crafting_shaped", "minecraft:crafting_shapeless")

# A vanilla 26.3 block table, for the shape this file insists on:
#   /home/adlf/Desktop/horizon/.local/horizon-client/26.3/data/minecraft/
#   loot_table/blocks/acacia_button.json
VANILLA_EXAMPLE = (
    "data/minecraft/loot_table/blocks/acacia_button.json in the prepared 26.3 resources"
)


def recipe_offenders(recipe: dict) -> list[str]:
    """Every ingredient of one recipe that is not a bare item id / #tag string."""
    kind = recipe.get("type", "")
    if kind not in CRAFTING:
        return []
    if kind == "minecraft:crafting_shaped":
        values = list(recipe.get("key", {}).values())
    else:
        values = list(recipe.get("ingredients", []))
    return [json.dumps(value) for value in values if not isinstance(value, str)]


def loot_offenders(table: dict) -> list[str]:
    """Every 1.21.1 pool of one loot table: its array condition, its `bonus_rolls`."""
    problems: list[str] = []
    for index, pool in enumerate(table.get("pools", [])):
        where = f"pool {index}"
        if "conditions" in pool:
            problems.append(f"{where}: \"conditions\": {json.dumps(pool['conditions'])} "
                            "(26.3 uses \"condition\": {...})")
        if "bonus_rolls" in pool:
            problems.append(f"{where}: \"bonus_rolls\": {json.dumps(pool['bonus_rolls'])} "
                            "(26.3 has no such key)")
        condition = pool.get("condition")
        if isinstance(condition, dict) and "condition" in condition and "type" not in condition:
            problems.append(f"{where}: condition spelled \"condition\": "
                            f"{json.dumps(condition['condition'])} (26.3 names it \"type\")")
        rolls = pool.get("rolls")
        if isinstance(rolls, float) and rolls.is_integer():
            problems.append(f"{where}: \"rolls\": {rolls} (26.3 spells an integral roll 1)")
    return problems


def check_recipe(name: str, parsed, bad: list[str]) -> None:
    for recipe in parsed if isinstance(parsed, list) else [parsed]:
        wrong = recipe_offenders(recipe)
        if wrong:
            bad.append(f"{name}: {recipe.get('type', '?')} ingredient(s) in the 1.21.1 "
                       "object form: " + ", ".join(wrong))
        elif recipe.get("type", "") in CRAFTING:
            print(f"  ok   {name}: {recipe['type'].split(':')[1]}, bare item/tag strings")


def check_loot(name: str, parsed, bad: list[str]) -> None:
    wrong = loot_offenders(parsed)
    if wrong:
        for problem in wrong:
            bad.append(f"{name}: 1.21.1 loot pool — {problem}")
    else:
        print(f"  ok   {name}: 26.3 loot pool (singular condition, no bonus_rolls, rolls 1)")


def read(name: str) -> list[tuple[str, str]]:
    """(path, text) for every recipe and loot table under a package or a data dir.

    The path is kept whole (not just the file name): the dialect a file is checked
    for follows from where it sits, and a bare `connector.json` cannot say whether
    it is a recipe or a loot table.
    """
    found: list[tuple[str, str]] = []
    if name.endswith(".hmod"):
        with zipfile.ZipFile(name) as archive:
            for entry in sorted(archive.namelist()):
                if entry.startswith("data/") and entry.endswith(".json") \
                        and ("/recipe/" in entry or "/loot_table/" in entry):
                    found.append((f"{Path(name).name}:{entry}", archive.read(entry).decode()))
        return found
    root = Path(name)
    files = sorted(root.rglob("*.json")) if root.is_dir() else [root]
    for path in files:
        text = str(path).replace("\\", "/")
        if "/recipe/" in text or "/loot_table/" in text:
            found.append((str(path), path.read_text()))
    return found


def main(argv: list[str]) -> int:
    targets = argv[1:] if argv and argv[0] == "--data" else argv
    if not targets:
        print("usage: data_shape.py <package.hmod> | --data <dir>", file=sys.stderr)
        return 64
    bad: list[str] = []
    checked = 0
    for target in targets:
        if target.endswith(".hmod") and not Path(target).is_file():
            print(f"missing package: {target}", file=sys.stderr)
            return 2
        for name, text in read(target):
            try:
                parsed = json.loads(text)
            except json.JSONDecodeError as error:
                bad.append(f"{name}: not JSON ({error})")
                continue
            checked += 1
            if "/loot_table/" in name:
                check_loot(name, parsed, bad)
            else:
                check_recipe(name, parsed, bad)
    if bad:
        for line in bad:
            print(f"  FAIL {line}", file=sys.stderr)
        print(f"data shape: {checked} file(s) checked, {len(bad)} 1.21.1 problem(s); "
              f"the shape to copy is {VANILLA_EXAMPLE}", file=sys.stderr)
        return 1
    print(f"data shape: {checked} recipe/loot file(s), every one in the 26.3 dialect")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
