//
//  PickerChoice.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// One entry of a picker.
public struct PickerChoice<Value: Hashable>: Hashable {

    /// The value the entry stands for.
    public let value: Value

    /// Whether the user can pick the entry. A value the mouse reports outside the offered list is
    /// shown, but never written.
    public let isSelectable: Bool

    /// Creates an entry.
    public init(value: Value, isSelectable: Bool) {
        self.value = value
        self.isSelectable = isSelectable
    }

    /// The entries of `options`, followed by `current` as a disabled entry when it is not listed.
    static func list(options: [Value], current: Value?) -> [Self] {
        var choices = options.map { Self(value: $0, isSelectable: true) }

        if let current,
           !options.contains(current) {
            choices.append(Self(value: current, isSelectable: false))
        }

        return choices
    }

}
