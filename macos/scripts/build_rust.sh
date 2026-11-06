#!/usr/bin/env bash
# Builds openatk-ffi for the host, generates the Swift bindings and the xcframework.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
macos="$root/macos"
bindings="$macos/Sources/OpenATKBindings"
headers="$macos/build/ffi-headers"
xcframework="$macos/Frameworks/OpenATKFFI.xcframework"

cd "$root"
# Match the package platform so the static library carries no newer minimum version.
export MACOSX_DEPLOYMENT_TARGET=14.0
cargo build -p openatk-ffi --release
library="$root/target/release/libopenatk_ffi.a"

rm -rf "$bindings" "$headers" "$xcframework"
mkdir -p "$bindings" "$headers"

cargo run -p openatk-ffi --release --bin uniffi-bindgen -- \
    generate --library "$root/target/release/libopenatk_ffi.dylib" \
    --language swift --out-dir "$bindings"

# The xcframework wants only the header and a module.modulemap.
mv "$bindings"/*.h "$headers/"
mv "$bindings"/*.modulemap "$headers/module.modulemap"

xcodebuild -create-xcframework \
    -library "$library" -headers "$headers" \
    -output "$xcframework"
