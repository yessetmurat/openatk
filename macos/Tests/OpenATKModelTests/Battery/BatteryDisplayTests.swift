//
//  BatteryDisplayTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

struct BatteryDisplayTests {

    @Test func chargingFlagHidesPercent() {
        #expect(SnapshotValue.sample(charging: true).batteryDisplay == .charging)
    }

    @Test func cableHidesPercentWithoutChargingFlag() {
        #expect(SnapshotValue.sample(onCable: true).batteryDisplay == .charging)
    }

    @Test func dongleNotChargingShowsPercent() {
        #expect(SnapshotValue.sample().batteryDisplay == .percent(87))
    }

}
