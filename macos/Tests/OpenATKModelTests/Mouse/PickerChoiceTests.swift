//
//  PickerChoiceTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import Testing

struct PickerChoiceTests {

    @Test(arguments: [
        (10, [10, 30, 60, 120]),
        (45, [30, 45, 60, 120]),
        (200, [30, 60, 120, 200]),
    ] as [(UInt16, [UInt16])])
    func offListValueSitsInOrder(current: UInt16, values: [UInt16]) {
        let choices = PickerChoice.list(options: [30, 60, 120], current: current)

        let disabled = choices.filter { !$0.isSelectable }.map(\.value)

        #expect(choices.map(\.value) == values)
        #expect(disabled == [current])
    }

    @Test func listedValueAddsNoEntry() {
        let choices = PickerChoice<UInt16>.list(options: [30, 60], current: 60)

        #expect(choices.map(\.value) == [30, 60])
        #expect(choices.map(\.isSelectable) == [true, true])
    }

}
