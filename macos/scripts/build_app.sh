#!/usr/bin/env bash
# Builds the Rust library, the Swift package and assembles macos/build/openatk.app.
set -euo pipefail

macos="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$macos/build/openatk.app"

"$macos/scripts/build_rust.sh"
swift build -c release --package-path "$macos"
binary="$(swift build -c release --package-path "$macos" --show-bin-path)/OpenATK"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/openatk"
cp "$macos/Resources/Info.plist" "$app/Contents/Info.plist"
mkdir "$app/Contents/Resources/fonts"
cp "$macos"/Resources/fonts/*.ttf "$macos/Resources/fonts/OFL.txt" "$app/Contents/Resources/fonts/"

# arm64 refuses unsigned code; an ad-hoc signature is enough.
codesign --force --sign - "$app"
