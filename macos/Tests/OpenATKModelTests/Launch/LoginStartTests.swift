//
//  LoginStartTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import CoreServices
import OpenATKModel
import Testing

struct LoginStartTests {

    private static let openApplication = AEEventID(kAEOpenApplication)
    private static let loginItem = OSType(keyAELaunchedAsLogInItem)

    @Test func openEventMarkedAsLoginItemIsALoginStart() {
        #expect(LoginStart.isLoginStart(eventID: Self.openApplication, launchKind: Self.loginItem))
    }

    @Test func plainOpenEventIsNotALoginStart() {
        #expect(!LoginStart.isLoginStart(eventID: Self.openApplication, launchKind: nil))
    }

    @Test func noEventIsNotALoginStart() {
        #expect(!LoginStart.isLoginStart(eventID: nil, launchKind: nil))
    }

    @Test func anotherEventIsNotALoginStart() {
        let reopen = AEEventID(kAEReopenApplication)

        #expect(!LoginStart.isLoginStart(eventID: reopen, launchKind: Self.loginItem))
    }

}
