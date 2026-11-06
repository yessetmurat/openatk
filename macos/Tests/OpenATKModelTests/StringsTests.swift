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

}
