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

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(windowWillClose),
            name: NSWindow.willCloseNotification,
            object: nil
        )
    }

    /// With the icon off there is no way back to a closed window, so the app quits.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        !preferences.menuBarIcon
    }

    /// With the icon off nothing polls in the background, so the app catches up here.
    func applicationDidBecomeActive(_ notification: Notification) {
        model.send(.refresh)
    }

    /// Opening the app again, from Finder or Spotlight, brings back a closed window: SwiftUI does
    /// not do it for a `Window` scene.
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows: Bool) -> Bool {
        if !hasVisibleWindows {
            showMainWindow()
        }

        return true
    }

    /// Brings the window forward. The Dock policy comes first: an accessory app takes no focus.
    /// Activation is the system's call. In a test with no user event `NSApp.activate()` was always
    /// refused, and this form usually accepted, so it is the one used here.
    func showMainWindow() {
        applyDockPolicy(windowOpen: true)
        openMainWindow?()
        NSApp.activate(ignoringOtherApps: true)
    }

    /// Takes the Dock icon away when the window closes. Only the main window counts: a sheet, a
    /// menu or a status item window cannot become main.
    @objc private func windowWillClose(_ notification: Notification) {
        guard let window = notification.object as? NSWindow,
              window.canBecomeMain else {
            return
        }

        applyDockPolicy(windowOpen: false)
    }

    private func applyDockPolicy(windowOpen: Bool) {
        let showsDockIcon = preferences.showsDockIcon(windowOpen: windowOpen)

        NSApp.setActivationPolicy(showsDockIcon ? .regular : .accessory)
    }

    /// Starts the mouse service. This is the one place that creates a `MouseClient`.
    private static func makeModel() -> MouseModel {
        let listener = MouseUpdateListener()
        let client = MouseClient(listener: listener)
        let model = MouseModel(sink: client)

        // Deliberate process-lifetime cycle: MouseClient -> listener -> MouseModel -> MouseClient.
        listener.model = model

        return model
    }

}
