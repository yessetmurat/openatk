//
//  SampleSnapshot.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import Foundation
import OpenATKBindings

extension SnapshotValue {

    /// A snapshot like the real mouse's, with the given rate and equal-axis stages.
    static func sample(
        profile: UInt8 = 1,
        pollingHz: UInt32 = 250,
        stages: [UInt32] = [400, 800, 1200, 1600],
        buttons: [ButtonValue] = [],
        charging: Bool = false,
        onCable: Bool = false
    ) -> Self {
        SnapshotValue(
            modelName: "Sample Mouse",
            profile: profile,
            profileCount: 4,
            percent: 87,
            charging: charging,
            onCable: onCable,
            pollingHz: pollingHz,
            pollingOptionsHz: [125, 250, 500, 1000],
            dpiMin: 50,
            dpiMax: 30000,
            sensor: .paw3950Ultra,
            currentStage: 1,
            stages: stages.map { StageValue(x: $0, y: $0) },
            buttons: buttons,
            firmware: "3.01",
            dongleFirmware: "1.0a",
            liftOffTenths: 7,
            liftOffMinTenths: 7,
            liftOffMaxTenths: 17,
            debounceMs: 0,
            debounceOptionsMs: Data([0, 1, 2, 4, 8, 15, 20]),
            motionSync: false,
            linearCorrection: false,
            rippleControl: false,
            sleepSeconds: 10,
            sleepOptionsSeconds: [30, 60, 120, 180, 300, 1200, 1500, 1800],
            angle: AngleValue(degrees: 0, enabled: false),
            angleMin: -30,
            angleMax: 30,
            farDistance: false
        )
    }

}
