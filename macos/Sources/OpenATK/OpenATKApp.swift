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

    @Environment(\.openWindow) private var openWindow

    var body: some Scene {
        @Bindable var preferences = delegate.preferences

        // From the app and not a view: with the icon off, no view may ever appear to hand it over.
        let _ = delegate.windowOpenerReady {
            openWindow(id: MainWindow.sceneID)
        }

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
