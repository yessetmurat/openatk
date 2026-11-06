//
//  StringsTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import Testing

struct StringsTests {

    @Test func profileCountsFromOne() {
        #expect(Strings.profile(index: 0) == "Profile 1")
    }

    @Test func profileDoesNotOverflowTheByte() {
        #expect(Strings.profile(index: 255) == "Profile 256")
    }

    @Test func profileTitleDoesNotOverflowTheByte() {
        #expect(Strings.profileTitle(index: 255) == "Profile: 256")
    }

    @Test(arguments: [
        (UInt16(10), "10 s"),
        (30, "30 s"),
        (60, "1 min"),
        (90, "90 s"),
        (120, "2 min"),
        (1800, "30 min"),
        (0, "0 s")
    ])
    func sleepTimeUsesSecondsOrWholeMinutes(seconds: UInt16, text: String) {
        #expect(Strings.sleepTime(seconds: seconds) == text)
    }

    @Test(arguments: [(UInt8(7), "0.7 mm"), (10, "1.0 mm"), (17, "1.7 mm")])
    func liftOffShowsMillimetres(tenths: UInt8, text: String) {
        #expect(Strings.liftOffHeight(tenths: tenths) == text)
    }

}
