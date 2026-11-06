//
//  KeyModifiers.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// The modifier keys held during a key press. The model owns this type so it stays free of
/// AppKit; the app converts from `NSEvent.ModifierFlags`.
public struct KeyModifiers: OptionSet, Sendable {

    public let rawValue: Int

    public init(rawValue: Int) {
        self.rawValue = rawValue
    }

    /// The Control key.
    public static let control = KeyModifiers(rawValue: 1 << 0)

    /// The Shift key.
    public static let shift = KeyModifiers(rawValue: 1 << 1)

    /// The Option (Alt) key.
    public static let option = KeyModifiers(rawValue: 1 << 2)

    /// The Command key.
    public static let command = KeyModifiers(rawValue: 1 << 3)

}
