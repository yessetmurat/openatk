//
//  MouseModel+Buttons.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

extension MouseModel {

    private static let fixedActions: [Action] = [
        .disabled, .mouseLeft, .mouseRight, .mouseMiddle, .mouseSide1, .mouseSide2,
        .dpiCycle, .dpiPlus, .dpiMinus, .wheelUp, .wheelDown,
    ]

    /// The actions a button can be given: one per profile of the mouse, among the fixed ones.
    public var actionChoices: [Action] {
        let profileCount = snapshot?.profileCount ?? 0
        let profiles = (0..<profileCount).map { Action.profile(index: $0) }

        return Self.fixedActions + profiles + [.profileCycle, .shortcut]
    }

    /// Whether the button's action can be changed here. `Other` actions keep their bytes.
    public func isEditable(button: MouseButton) -> Bool {
        guard let action = snapshot?.buttons.first(where: { $0.button == button })?.action else {
            return false
        }

        if case .other = action {
            return false
        }

        return true
    }

    /// Gives `button` a non-shortcut action, unless it already has it.
    public func selectAction(button: MouseButton, action: Action) {
        guard let current = snapshot?.buttons.first(where: { $0.button == button }),
              current.action != action else {
            return
        }

        send(.setButton(button: button, action: action))
    }

    /// Stores a shortcut on `button`, unless it is already stored. Refuses any shape the mouse
    /// cannot hold: one key, or one or two modifiers plus one key.
    public func setShortcut(button: MouseButton, keys: [ShortcutKeyValue]) {
        guard KeyUsage.isValidShortcut(keys) else {
            return
        }

        let stored = snapshot?.buttons.first(where: { $0.button == button })

        guard stored?.action != .shortcut || stored?.keys != keys else {
            return
        }

        send(.setShortcut(button: button, keys: keys))
    }

    /// The text of the button's stored shortcut, such as "Ctrl+C", or the unknown-shortcut label.
    public func shortcutText(button: MouseButton) -> String {
        guard let keys = snapshot?.buttons.first(where: { $0.button == button })?.keys,
              !keys.isEmpty else {
            return Strings.unknownShortcut
        }

        return keys.map(KeyUsage.text).joined(separator: "+")
    }

    /// Opens key capture on `button`.
    public func beginCapture(button: MouseButton) {
        guard !isPending else {
            return
        }

        captureButton = button
        captureMessage = nil
    }

    /// Closes key capture.
    public func cancelCapture() {
        captureButton = nil
        captureMessage = nil
    }

    /// Takes a key press during capture. An auto-repeat is ignored, so a key held down before the
    /// capture opened is not taken. Returns the shortcut and closes the capture, or sets
    /// ``captureMessage`` and stays in capture when the key has no usage or the shortcut would
    /// have more than two modifiers.
    public func captureKey(
        code: UInt16,
        modifiers: KeyModifiers,
        isRepeat: Bool
    ) -> [ShortcutKeyValue]? {
        guard captureButton != nil,
              !isRepeat else {
            return nil
        }

        let held = KeyUsage.modifiers(modifiers)

        guard let usage = KeyUsage.fromKeyCode(code) else {
            captureMessage = Strings.unmappedKey
            return nil
        }

        guard held.count <= 2 else {
            captureMessage = Strings.tooManyModifiers
            return nil
        }

        cancelCapture()

        return held + [.general(usage: usage)]
    }

    /// Asks for a change to `button`. Returns `true` and holds it in ``confirmation`` when it
    /// would leave no left-click button; otherwise sends it. Closes key capture either way.
    @discardableResult
    public func request(button: MouseButton, change: ButtonRequest.Change) -> Bool {
        cancelCapture()

        guard let buttons = snapshot?.buttons else {
            return false
        }

        let action: Action
        switch change {
        case .action(let chosen):
            action = chosen
        case .shortcut:
            action = .shortcut
        }

        let request = ButtonRequest(button: button, change: change)

        guard ButtonRules.losesPrimaryClick(buttons: buttons, button: button, action: action) else {
            confirm(request)
            return false
        }

        confirmation = request
        return true
    }

    /// Sends a request the user confirmed.
    public func confirm(_ request: ButtonRequest) {
        confirmation = nil

        switch request.change {
        case .action(let action):
            selectAction(button: request.button, action: action)
        case .shortcut(let keys):
            setShortcut(button: request.button, keys: keys)
        }
    }

    /// Drops the held request.
    public func cancelConfirmation() {
        confirmation = nil
    }

}
