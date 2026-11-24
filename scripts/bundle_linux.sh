#!/bin/sh
# Build a Linux release and pack it as dist/VoidArmada-linux-x86_64.tar.gz:
# the void_armada binary with the assets folder beside it. Extract anywhere
# and run ./void_armada.
#
# Build deps (Debian/Ubuntu):
#   apt install pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
set -eu
cd "$(dirname "$0")/.."
cargo build --release
NAME=VoidArmada-linux-x86_64
OUT=dist/$NAME
rm -rf "$OUT" "$OUT.tar.gz"
mkdir -p "$OUT/assets"
cp target/release/void_armada "$OUT/void_armada"
# the game reads sprites, sounds and fonts at runtime (the icon is compiled in)
cp -R assets/sprites assets/sfx assets/fonts "$OUT/assets/"
cp LICENSE "$OUT/LICENSE"
tar -C dist -czf "$OUT.tar.gz" "$NAME"
echo "wrote $OUT.tar.gz"
