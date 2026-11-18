#!/usr/bin/env bash
# The SDK ladder for this port, inside one build-slot lease:
#
#   ports/iceandfire/tools/check.sh
#
# The launcher builds the package from `horizon-mod.toml`; this script only proves the
# crate compiles against the pinned SDK, then that the package is well formed and still
# fits the build it declares. It runs from the port directory (the slot runner starts in
# the campaign root), and it never passes `--target-dir`.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
campaign=$(cd ../.. && pwd)

status=0
step() {
    echo "== $*"
    "$@" || status=1
}

step env CARGO_INCREMENTAL=0 cargo test --offline
step env CARGO_INCREMENTAL=0 cargo clippy --offline --all-targets -- -D warnings
step env CARGO_INCREMENTAL=0 cargo horizon-mod build
# The deliverable is the port root's copy of what was just built: `check` and `fits`
# below must see the bytes this run produced, not a previous run's.
step cp target/horizon-mod/iceandfire.hmod ./iceandfire.hmod
step cargo horizon-mod check iceandfire.hmod
step cargo horizon-mod fits iceandfire.hmod
sha256sum iceandfire.hmod
echo "CHECK_EXIT:$status"
exit $status
