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

    /// The entries of `options`, in ascending order, with `current` in its place among them as a
    /// disabled entry when it is not listed.
    public static func list(options: [Value], current: Value?) -> [Self] where Value: Comparable {
        var choices = options.map { Self(value: $0, isSelectable: true) }

        if let current,
           !options.contains(current) {
            let index = options.firstIndex { $0 > current } ?? options.count

            choices.insert(Self(value: current, isSelectable: false), at: index)
        }

        return choices
    }

}
