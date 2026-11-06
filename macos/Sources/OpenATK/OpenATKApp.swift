//
//  OpenATKApp.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

@main
struct OpenATKApp: App {

    @NSApplicationDelegateAdaptor private var delegate: AppDelegate

    var body: some Scene {
        @Bindable var preferences = delegate.preferences

        Window(Strings.appName, id: MainWindow.sceneID) {
            MainWindow(
                model: delegate.model,
                preferences: preferences,
                launchAtLogin: delegate.launchAtLogin
            )
        }
        .windowResizability(.contentSize)
        // The app delegate decides at launch whether the window opens.
        .defaultLaunchBehavior(.suppressed)
        // Restoration brings the window back past `.suppressed`, also at a login start.
        .restorationBehavior(.disabled)

        MenuBarExtra(isInserted: $preferences.menuBarIcon) {
            MenuBarContent(delegate: delegate)
        } label: {
            MenuBarLabel(delegate: delegate)
        }
        .menuBarExtraStyle(.menu)
    }

}
