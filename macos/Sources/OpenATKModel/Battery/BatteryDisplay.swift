//
//  BatteryDisplay.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// What to show for the battery. On the cable the mouse reports an inflated percent, so a
/// charging or cabled mouse shows no number.
public enum BatteryDisplay: Equatable {

    case charging
    case percent(UInt8)

}

extension SnapshotValue {

    /// The battery as shown to the user: charging with no number, or the percent.
    public var batteryDisplay: BatteryDisplay {
        charging || onCable ? .charging : .percent(percent)
    }

}
