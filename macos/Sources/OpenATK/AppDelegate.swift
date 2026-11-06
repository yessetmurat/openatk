//
//  AppDelegate.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import AppKit
import OpenATKBindings
import OpenATKModel

/// Owns the model and the preferences, so they work with no window open.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {

    let preferences = Preferences(defaults: .standard)
    let model = AppDelegate.makeModel()

    /// Opens the window. Only a view can reach `openWindow`, so `MenuBarLabel` sets this. Without
    /// the menu bar icon closing the window quits, so no reopen depends on it.
    var openMainWindow: (() -> Void)?

    func applicationDidFinishLaunching(_ notification: Notification) {
        preferences.applyBackgroundPolling(to: model)
    }

    /// With the icon off there is no way back to a closed window, so the app quits.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        !preferences.menuBarIcon
    }

    /// With the icon off nothing polls in the background, so the app catches up here.
    func applicationDidBecomeActive(_ notification: Notification) {
        model.send(.refresh)
    }

    /// A click on the Dock icon brings back a closed window: SwiftUI does not do it for a `Window`
    /// scene.
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows: Bool) -> Bool {
        if !hasVisibleWindows {
            openMainWindow?()
        }

        return true
    }

    /// Starts the mouse service. This is the one place that creates a `MouseClient`.
    private static func makeModel() -> MouseModel {
        let listener = MouseUpdateListener()
        let client = MouseClient(listener: listener)
        let model = MouseModel(sink: client)

        listener.model = model

        return model
    }

}
