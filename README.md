# openatk

A light native replacement for ATK HUB for daily use, for the ATK F1 EXTREME mouse over its 2.4 GHz dongle or the cable.

It changes DPI and polling rate, remaps buttons (including keyboard keys), switches profiles, shows the battery level, and lives in the menu bar, with submenus for DPI, polling rate and profile. It also sets lift-off, debounce, motion sync, ripple control, angle, linear correction, sleep timer and far distance, and can launch at login. No browser, account, cloud or network access.

Editing applies to the active profile only: switch to a profile to change it.

Status: macOS only for now. Windows and Linux clients are planned over the same `core`. Launch at login is not yet verified across a real logout.

openatk is not affiliated with ATK. Firmware updates stay in ATK HUB: openatk never sends upgrade commands.

## Devices

| Device | Supported | Tested |
|---|:---:|:---:|
| ATK F1 EXTREME (2.4 GHz dongle and cable) | ✅ | ✅ |
| Other ATK and VXE mice | ❌ | ❌ |

Other models share the protocol, but openatk refuses to change their settings until they are added and tested.

## Layout

- `core`: Rust protocol library and device service.
- `ffi`: uniffi layer over `core`.
- `macos`: SwiftUI client with a window and a menu bar icon.

## Building

You need a Rust toolchain and Xcode.

The Swift bindings and the framework are generated and ignored by git, so on a fresh checkout run this before the tests:

```sh
macos/scripts/build_rust.sh
swift test --package-path macos
```

Build the app:

```sh
macos/scripts/build_app.sh
```

The result is `macos/build/openatk.app`.

### Signing

macOS binds the Input Monitoring grant to the app signature. An ad-hoc signature (the default) changes at every build, so the grant is lost each time. To keep it, sign with a stable identity:

```sh
OPENATK_SIGN_IDENTITY="<signing identity>" macos/scripts/build_app.sh
```

If the grant is stale after a change of signature:

```sh
tccutil reset ListenEvent app.openatk
```

### Release

`macos/scripts/package_release.sh` builds `macos/build/openatk-<version>.zip`, signed for distribution. Set `OPENATK_RELEASE_IDENTITY` to a Developer ID Application identity. To notarize too, save a profile once with `xcrun notarytool store-credentials <profile>` and set `OPENATK_NOTARY_PROFILE` to it.

A pushed tag `v<x.y.z>` runs `.github/workflows/release.yml`: it builds, signs, notarizes, attaches the zip to a GitHub release and writes the cask in the Homebrew tap. The workflow file lists the secrets it needs.

## Permissions

The app needs Input Monitoring, because the device exposes keyboard collections. `settings_tool` run from a terminal uses the terminal's Input Monitoring grant, not the app's.

### macOS version

Built for macOS 15 and later, but tested only on macOS 27.

## Settings tool

A development tool to back up and inspect the mouse settings:

```sh
cargo run -p openatk-core --example settings_tool -- dump <file>
cargo run -p openatk-core --example settings_tool -- restore <file>
cargo run -p openatk-core --example settings_tool -- watch
cargo run -p openatk-core --example settings_tool -- set-polling <hz>
```

Reads and writes need an awake mouse: when it sleeps, `settings_tool` prints "the mouse is not connected".

Dump the settings before experimenting. `dump` and `restore` cover the active profile only. `dump` refuses to overwrite an existing file. `restore` writes every line of the file that lies inside the writable ranges, so edit a dump with care.

## Safety

openatk reads a setting back after each write and shows an error if it differs. To recover a misconfigured mouse, use `restore` or ATK HUB.

## Fonts

IBM Plex Sans, under the SIL Open Font License. See `macos/Resources/fonts`.

## Licence

MIT, see `LICENSE`. IBM Plex Sans keeps its own licence.
