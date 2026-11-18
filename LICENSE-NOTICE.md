# Licence notice — Ice and Fire Community Edition, Horizon port

This port is a **derivative work** of **Ice And Fire Community Edition** (`iceandfire`),
version **2.0-beta.17**, and is distributed under the **same licence, LGPL-3.0**.

## The original

* Name: Ice And Fire Community Edition ("An unofficial fork of Ice And Fire")
* Mod id: `iceandfire`
* Version ported: `2.0-beta.17` (the jar shipped by All the Mods 10)
* Authors, as the jar's `META-INF/neoforge.mods.toml` names them: **IAFEnvoy, xiaowu, uoay**
  (`credits = "Alexthe666"`, the original Ice and Fire author)
* Upstream repository: <https://github.com/IAFEnvoy/IceAndFire-CE>
* Upstream licence file: <https://github.com/IAFEnvoy/IceAndFire-CE/blob/main/LICENSE>
  (`Copyright (c) 2024 IAFEnvoy`, GNU Lesser General Public License v3.0, which
  incorporates the GNU General Public License v3.0)
* Jar this port was written against:
  `iceandfire-2.0-beta.17.jar`, 20,848,811 bytes, sha256
  `ee52349445615417e69ab64cf15dbb96b9332a9117225119d281695c5f8d90f0`
  (read from the pack; kept as evidence, never rewritten)
* Licence read from: the jar's own `META-INF/neoforge.mods.toml` (`license = "LGPL-3.0"`);
  the licence text comes from the upstream repository's `LICENSE`

## This port

* A clean-room rewrite in Rust for [Horizon](https://github.com/horizon-rust/horizon)
  (Minecraft Java 26.3). No Java source was copied, translated or decompiled; the
  behaviour was read from the mod's public source repository and its own data files,
  and implemented against Horizon's stamped hook catalogue (`ports/iceandfire/FINDINGS.md`
  records what was read, file and line).
* **Licence: LGPL-3.0**, as the original. The complete corresponding source of this
  port is its repository: `Cargo.toml`, `mixins/iceandfire/**`, `content/**` and
  `tools/**`. `LICENSE` (beside this file) carries the LGPL-3.0 and GPL-3.0 texts, and
  both it and this notice travel inside the distributed `.hnmod`.
* The carried assets are the mod's own (`assets/iceandfire/textures/item/dragonegg_red.png`,
  the `dragonegg_red` item model), redistributed under the same LGPL-3.0.
* Modified from the original: this is a re-implementation for a different engine, not a
  patch to the original. The original's copyright and licence notices are preserved here
  and in `LICENSE`; the port's own changes are summarised in `README.md` and `notes.md`.

## What a redistributor must do

Keep `LICENSE`, this notice and the complete source available, and ship any modified
version under the LGPL-3.0 as well. Publishing this port needs the maintainer's
approval (campaign rule, `ARCHITECTURE.md` §8); it is **not** an All-Rights-Reserved
port and may be published under these terms.
