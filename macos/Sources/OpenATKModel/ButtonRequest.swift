//
//  ButtonRequest.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// A change to one button that the user asked for.
public struct ButtonRequest: Equatable, Sendable {

    /// What the button should become.
    public enum Change: Equatable, Sendable {

        /// A non-shortcut action.
        case action(Action)

        /// A keyboard shortcut.
        case shortcut([ShortcutKeyValue])

    }

    /// The button to change.
    public let button: MouseButton

    /// The change.
    public let change: Change

    /// Creates a request.
    public init(button: MouseButton, change: Change) {
        self.button = button
        self.change = change
    }

}
