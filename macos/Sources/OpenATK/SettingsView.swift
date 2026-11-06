//
//  SettingsView.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The app's own settings. Nothing here touches the mouse.
struct SettingsView: View {

    @Bindable var preferences: Preferences

    var body: some View {
        Form {
            Section {
                Toggle(Strings.showMenuBarIcon, isOn: $preferences.menuBarIcon)
                LabeledContent(Strings.version, value: Self.version)
            }
        }
        .formStyle(.grouped)
    }

    private static var version: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
            ?? Strings.noValue
    }

}
