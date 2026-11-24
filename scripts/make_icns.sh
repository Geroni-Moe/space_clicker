#!/bin/sh
# Pack assets/icon/icon_*.png (drawn by art/scripts/icon.lua) into
# assets/icon/VoidArmada.icns with macOS's iconutil.
set -eu
cd "$(dirname "$0")/.."
ICONSET=target/VoidArmada.iconset
rm -rf "$ICONSET"
mkdir -p "$ICONSET"
src=assets/icon
cp "$src/icon_16.png" "$ICONSET/icon_16x16.png"
cp "$src/icon_32.png" "$ICONSET/icon_16x16@2x.png"
cp "$src/icon_32.png" "$ICONSET/icon_32x32.png"
cp "$src/icon_64.png" "$ICONSET/icon_32x32@2x.png"
cp "$src/icon_128.png" "$ICONSET/icon_128x128.png"
cp "$src/icon_256.png" "$ICONSET/icon_128x128@2x.png"
cp "$src/icon_256.png" "$ICONSET/icon_256x256.png"
cp "$src/icon_512.png" "$ICONSET/icon_256x256@2x.png"
cp "$src/icon_512.png" "$ICONSET/icon_512x512.png"
cp "$src/icon_1024.png" "$ICONSET/icon_512x512@2x.png"
iconutil -c icns "$ICONSET" -o "$src/VoidArmada.icns"
echo "wrote $src/VoidArmada.icns"
