//
//  FieldDraft.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// Text typed into a numeric field and not yet confirmed by the mouse.
public struct FieldDraft {

    /// A numeric field that holds its typed text in the model.
    public enum Field: Equatable {

        /// The DPI of the stage at this index, counted from 0.
        case stage(Int)

        /// The sensor angle.
        case angle

    }

    let field: Field
    let text: String

    /// Whether this draft's write is in flight: the field keeps showing it until the mouse
    /// answers, so it does not flash the old value, and it is never written twice.
    let isSent: Bool

}
