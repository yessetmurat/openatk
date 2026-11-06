//
//  SampleSnapshot.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

extension SnapshotValue {

    /// A snapshot like the real mouse's, with the given rate and equal-axis stages.
    static func sample(pollingHz: UInt32 = 250, stages: [UInt32] = [400, 800, 1200, 1600]) -> Self {
        SnapshotValue(
            modelName: "Sample Mouse",
            profile: 1,
            percent: 87,
            charging: false,
            pollingHz: pollingHz,
            pollingOptionsHz: [125, 250, 500, 1000],
            dpiMin: 50,
            dpiMax: 30000,
            sensor: .paw3950Ultra,
            currentStage: 1,
            stages: stages.map { StageValue(x: $0, y: $0) },
            buttons: []
        )
    }

}
