//
//  SnapshotValue+Profiles.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

extension SnapshotValue {

    /// The entries of a profile picker. The mouse's byte is passed on unchecked, so an active
    /// profile outside the count shows as itself, disabled.
    public var profileChoices: [PickerChoice<UInt8>] {
        PickerChoice.list(options: Array(0..<profileCount), current: profile)
    }

}
