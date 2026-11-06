//
//  LaunchWindowTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import Testing

struct LaunchWindowTests {

    @Test func normalStartOpensOnceWhenTheClosureComesFirst() {
        var window = LaunchWindow()

        let onClosure = window.closureReady()
        let onDecision = window.decide(isLoginStart: false, menuBarIcon: true)

        #expect(!onClosure)
        #expect(onDecision)
    }

    @Test func normalStartOpensOnceWhenTheDecisionComesFirst() {
        var window = LaunchWindow()

        let onDecision = window.decide(isLoginStart: false, menuBarIcon: true)
        let onClosure = window.closureReady()

        #expect(!onDecision)
        #expect(onClosure)
    }

    /// `onAppear` can fire again, and must not bring back a window the user closed.
    @Test func normalStartOpensOnlyOnce() {
        var window = LaunchWindow()
        _ = window.closureReady()
        _ = window.decide(isLoginStart: false, menuBarIcon: true)

        let again = window.closureReady()

        #expect(!again)
    }

    @Test func loginStartWithIconOpensNothing() {
        var window = LaunchWindow()

        let onDecision = window.decide(isLoginStart: true, menuBarIcon: true)
        let onClosure = window.closureReady()

        #expect(!onDecision)
        #expect(!onClosure)
        #expect(!window.wantsWindow)
    }

    @Test func loginStartWithoutIconOpensWindow() {
        var window = LaunchWindow()

        let onDecision = window.decide(isLoginStart: true, menuBarIcon: false)
        let onClosure = window.closureReady()

        #expect(!onDecision)
        #expect(onClosure)
        #expect(window.wantsWindow)
    }

    @Test func normalStartWithoutIconOpensWindow() {
        var window = LaunchWindow()
        _ = window.closureReady()

        let onDecision = window.decide(isLoginStart: false, menuBarIcon: false)

        #expect(onDecision)
    }

    @Test func nothingOpensBeforeTheDecision() {
        var window = LaunchWindow()

        let onClosure = window.closureReady()

        #expect(!onClosure)
        #expect(!window.wantsWindow)
    }

}
