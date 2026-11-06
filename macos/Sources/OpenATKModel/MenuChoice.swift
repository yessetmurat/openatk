//
//  MenuChoice.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// A checkable menu item.
public struct MenuChoice: Equatable {

    /// What a pick selects.
    public enum Target: Equatable {

        /// The DPI stage at `index`.
        case stage(index: Int)

        /// A polling rate.
        case rate(hertz: UInt32)

    }

    /// The item's text.
    public let title: String

    /// What a pick selects.
    public let target: Target

    /// Whether the mouse reports this as the current value.
    public let isChecked: Bool

    /// Whether the item can be picked.
    public let isEnabled: Bool

    /// Creates a choice, enabled unless stated.
    public init(
        title: String,
        target: Target,
        isChecked: Bool,
        isEnabled: Bool = true
    ) {
        self.title = title
        self.target = target
        self.isChecked = isChecked
        self.isEnabled = isEnabled
    }

}
