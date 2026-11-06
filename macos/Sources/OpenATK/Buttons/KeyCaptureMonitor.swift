//
//  KeyCaptureMonitor.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import AppKit
import OpenATKBindings
import OpenATKModel

/// Watches key presses while a capture is open, and swallows them so typing reaches nothing else.
@MainActor
final class KeyCaptureMonitor {

    private static let escapeKeyCode: UInt16 = 53

    private var monitor: Any?

    /// Starts watching. `onKey` receives the button and shortcut once a valid key is pressed.
    func start(model: MouseModel, onKey: @escaping @MainActor (MouseButton, [ShortcutKeyValue]) -> Void) {
        stop()

        monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            MainActor.assumeIsolated {
                guard let button = model.captureButton else {
                    return
                }

                if event.keyCode == Self.escapeKeyCode {
                    model.cancelCapture()
                } else if let keys = model.captureKey(
                    code: event.keyCode,
                    modifiers: KeyModifiers(event.modifierFlags),
                    isRepeat: event.isARepeat
                ) {
                    onKey(button, keys)
                }
            }

            return nil
        }
    }

    /// Stops watching.
    func stop() {
        if let monitor {
            NSEvent.removeMonitor(monitor)
        }

        monitor = nil
    }

}

private extension KeyModifiers {

    init(_ flags: NSEvent.ModifierFlags) {
        var modifiers: KeyModifiers = []

        if flags.contains(.control) { modifiers.insert(.control) }
        if flags.contains(.shift) { modifiers.insert(.shift) }
        if flags.contains(.option) { modifiers.insert(.option) }
        if flags.contains(.command) { modifiers.insert(.command) }

        self = modifiers
    }

}
