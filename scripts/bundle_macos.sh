#!/bin/sh
# Build a release "Void Armada.app" in dist/: the binary in Contents/MacOS,
# the game's assets in Contents/Resources, and the app icon. Unsigned; for
# running locally. Usage: scripts/bundle_macos.sh
set -eu
cd "$(dirname "$0")/.."
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
scripts/make_icns.sh
cargo build --release
APP="dist/Void Armada.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/void_armada "$APP/Contents/MacOS/void_armada"
cp -R assets "$APP/Contents/Resources/assets"
cp assets/icon/VoidArmada.icns "$APP/Contents/Resources/VoidArmada.icns"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Void Armada</string>
  <key>CFBundleDisplayName</key><string>Void Armada</string>
  <key>CFBundleIdentifier</key><string>dev.voidarmada.game</string>
  <key>CFBundleExecutable</key><string>void_armada</string>
  <key>CFBundleIconFile</key><string>VoidArmada</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.games</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST
echo "built $APP"
