#!/usr/bin/env bash
# Builds a signed release zip of openatk, and notarizes it when a notary profile is given.
#
# OPENATK_RELEASE_IDENTITY  required, a "Developer ID Application" signing identity
# OPENATK_NOTARY_PROFILE    optional, a profile saved with `xcrun notarytool store-credentials`
# OPENATK_NOTARY_KEYCHAIN   optional, the keychain holding that profile, for CI
set -euo pipefail

: "${OPENATK_RELEASE_IDENTITY:?set OPENATK_RELEASE_IDENTITY to a Developer ID Application identity}"

macos="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app="$macos/build/openatk.app"
version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$macos/Resources/Info.plist")"
archive="$macos/build/openatk-$version.zip"

OPENATK_SIGN_IDENTITY="$OPENATK_RELEASE_IDENTITY" "$macos/scripts/build_app.sh"

# Notarization needs the hardened runtime and a secure timestamp.
codesign --force --options runtime --timestamp --sign "$OPENATK_RELEASE_IDENTITY" "$app"
codesign --verify --strict --verbose=1 "$app"

rm -f "$archive"
ditto -c -k --keepParent "$app" "$archive"

if [ -z "${OPENATK_NOTARY_PROFILE:-}" ]; then
    echo "Signed, not notarized: $archive"
    exit 0
fi

keychain_option=()
if [ -n "${OPENATK_NOTARY_KEYCHAIN:-}" ]; then
    keychain_option=(--keychain "$OPENATK_NOTARY_KEYCHAIN")
fi

xcrun notarytool submit "$archive" --keychain-profile "$OPENATK_NOTARY_PROFILE" ${keychain_option[@]+"${keychain_option[@]}"} --wait
xcrun stapler staple "$app"

# Zip again, so the archive holds the stapled ticket.
rm -f "$archive"
ditto -c -k --keepParent "$app" "$archive"
spctl --assess --type execute --verbose=2 "$app"
echo "Notarized: $archive"
