#!/bin/sh
# Cross-compile a Windows release build and pack it as dist/VoidArmada-windows.zip:
# VoidArmada.exe (icon embedded by build.rs) with the assets folder beside it.
# Unzip anywhere on a Windows PC and run VoidArmada.exe.
#
# Needs the MinGW-w64 toolchain and the Rust target:
#   brew install mingw-w64   (Debian/Ubuntu: apt install mingw-w64)
#   rustup target add x86_64-pc-windows-gnu
set -eu
cd "$(dirname "$0")/.."
TARGET=x86_64-pc-windows-gnu
cargo build --release --target "$TARGET"
OUT=dist/VoidArmada-windows
rm -rf "$OUT" "$OUT.zip"
mkdir -p "$OUT"
cp "target/$TARGET/release/void_armada.exe" "$OUT/VoidArmada.exe"
# the game reads sprites, sounds and fonts at runtime (the icon is compiled in)
mkdir -p "$OUT/assets"
cp -R assets/sprites assets/sfx assets/fonts "$OUT/assets/"
cp LICENSE "$OUT/LICENSE.txt"
(cd dist && zip -qr VoidArmada-windows.zip VoidArmada-windows)
echo "wrote $OUT.zip"
