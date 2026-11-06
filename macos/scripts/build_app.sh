#!/usr/bin/env bash
# Builds the Rust library, the Swift package and assembles macos/build/openatk.app.
set -euo pipefail

macos="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$macos/build/openatk.app"

"$macos/scripts/build_rust.sh"
# SwiftPM stamps the deployment target as the linked SDK version, and macOS then gives the app the
# old look instead of Liquid Glass. Pass the real SDK version to the linker.
linker_sdk=(-Xlinker -platform_version -Xlinker macos -Xlinker 15.0 -Xlinker "$(xcrun --show-sdk-version)")
swift build -c release --package-path "$macos" "${linker_sdk[@]}"
binary="$(swift build -c release --package-path "$macos" "${linker_sdk[@]}" --show-bin-path)/OpenATK"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/openatk"
cp "$macos/Resources/Info.plist" "$app/Contents/Info.plist"
mkdir "$app/Contents/Resources/fonts"
cp "$macos"/Resources/fonts/*.ttf "$macos/Resources/fonts/OFL.txt" "$app/Contents/Resources/fonts/"

# arm64 refuses unsigned code. An ad-hoc signature is enough to run, but its identity is the
# cdhash, so every rebuild loses the Input Monitoring grant. Set OPENATK_SIGN_IDENTITY to a
# stable signing identity to keep the grant across rebuilds.
if [ -n "${OPENATK_SIGN_IDENTITY:-}" ]; then
    echo "Signing with identity: $OPENATK_SIGN_IDENTITY"
    codesign --force --sign "$OPENATK_SIGN_IDENTITY" "$app"
else
    echo "Signing ad-hoc (OPENATK_SIGN_IDENTITY is not set)"
    codesign --force --sign - "$app"
fi
