//
//  AppDelegate.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import AppKit
import OpenATKBindings
import OpenATKModel
import ServiceManagement

/// Owns the model and the preferences, so they work with no window open.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {

    let preferences = Preferences(defaults: .standard)
    let model = AppDelegate.makeModel()

    let launchAtLogin = LaunchAtLoginSetting(service: SMAppService.mainApp)

    /// Opens the window. Only SwiftUI can reach `openWindow`, so `OpenATKApp` sets this.
    private var openMainWindow: (() -> Void)?

    private var launchWindow = LaunchWindow()
    private var mainWindowClosed = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        preferences.applyBackgroundPolling(to: model)

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(windowWillClose),
            name: NSWindow.willCloseNotification,
            object: nil
        )

        let opensNow = launchWindow.decide(
            isLoginStart: isLoginStart,
            menuBarIcon: preferences.menuBarIcon
        )

        applyDockPolicy(windowOpen: launchWindow.wantsWindow)

        if opensNow {
            showMainWindow()
        }
    }

    /// Takes the action that opens the window, and uses it if the launch was waiting for it.
    func windowOpenerReady(_ open: @escaping () -> Void) {
        openMainWindow = open

        if launchWindow.closureReady() {
            showMainWindow()
        }
    }

    /// With the icon off there is no way back to a closed window, so the app quits.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        preferences.quitsAfterLastWindowClosed(mainWindowClosed: mainWindowClosed)
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

        mainWindowClosed = true
        applyDockPolicy(windowOpen: false)
    }

    /// Whether the app was started as a login item. The open event is current only while
    /// `applicationDidFinishLaunching` runs: it is `nil` in `applicationWillFinishLaunching` and
    /// when the scenes are built.
    private var isLoginStart: Bool {
        let event = NSAppleEventManager.shared().currentAppleEvent

        return LoginStart.isLoginStart(
            eventID: event?.eventID,
            launchKind: event?.paramDescriptor(forKeyword: keyAEPropData)?.enumCodeValue
        )
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
