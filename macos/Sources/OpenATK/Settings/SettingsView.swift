//
//  SettingsView.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The app's own settings. Nothing here writes to the mouse.
struct SettingsView: View {

    let model: MouseModel
    @Bindable var preferences: Preferences
    let launchAtLogin: LaunchAtLoginSetting

    @Environment(\.scenePhase) private var scenePhase

    var body: some View {
        Form {
            Section {
                Toggle(Strings.showMenuBarIcon, isOn: $preferences.menuBarIcon)
                launchToggle
                LabeledContent(Strings.version, value: Self.version)

                if let firmware = model.snapshot?.firmware {
                    LabeledContent(Strings.firmware, value: firmware)
                }

                if let dongleFirmware = model.snapshot?.dongleFirmware {
                    LabeledContent(Strings.dongleFirmware, value: dongleFirmware)
                }
            } footer: {
                if launchAtLogin.needsApproval {
                    Text(Strings.launchAtLoginNeedsApproval)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .formStyle(.grouped)
        .onAppear {
            launchAtLogin.refresh()
        }
        // The user approves or removes the item in System Settings, and comes back here.
        .onChange(of: scenePhase) {
            if scenePhase == .active {
                launchAtLogin.refresh()
            }
        }
    }

    private var launchToggle: some View {
        let isOn = Binding(
            get: { launchAtLogin.isEnabled },
            set: { launchAtLogin.set(enabled: $0) }
        )

        // Safe only while no other row of this section has an `.id`: see `SensorView.rowID`.
        return Toggle(Strings.launchAtLogin, isOn: isOn)
            .id(launchAtLogin.refreshCount)
    }

    private static var version: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
            ?? Strings.noValue
    }

}
