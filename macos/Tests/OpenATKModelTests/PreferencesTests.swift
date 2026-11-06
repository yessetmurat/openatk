//
//  PreferencesTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import Foundation
import OpenATKBindings
import OpenATKModel
import Testing

/// Serialized, because all tests share one suite on disk: removing a domain leaves an empty
/// file behind, so a suite per test would add a file to the user's preferences on every run.
@MainActor
@Suite(.serialized)
struct PreferencesTests {

    private static let suiteName = "openatk-PreferencesTests"

    private let defaults: UserDefaults

    init() throws {
        defaults = try #require(UserDefaults(suiteName: Self.suiteName))
        defaults.removePersistentDomain(forName: Self.suiteName)
    }

    @Test func defaultIsOn() {
        let preferences = Preferences(defaults: defaults)

        #expect(preferences.menuBarIcon)
    }

    @Test func roundTripsThroughDefaults() {
        let preferences = Preferences(defaults: defaults)

        preferences.menuBarIcon = false

        #expect(!Preferences(defaults: defaults).menuBarIcon)

        preferences.menuBarIcon = true

        #expect(Preferences(defaults: defaults).menuBarIcon)
    }

    @Test func pollingFollowsThePreference() {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        let preferences = Preferences(defaults: defaults)

        preferences.applyBackgroundPolling(to: model)
        preferences.menuBarIcon = false
        preferences.menuBarIcon = true

        #expect(sink.commands == [
            .setBackgroundPolling(enabled: true),
            .setBackgroundPolling(enabled: false),
            .setBackgroundPolling(enabled: true),
        ])
    }

    @Test func writingTheSameValueSendsNothing() {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        let preferences = Preferences(defaults: defaults)
        preferences.applyBackgroundPolling(to: model)

        preferences.menuBarIcon = true

        #expect(sink.commands == [.setBackgroundPolling(enabled: true)])
    }

    @Test func savedOffTurnsPollingOffAtLaunch() {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        Preferences(defaults: defaults).menuBarIcon = false

        Preferences(defaults: defaults).applyBackgroundPolling(to: model)

        #expect(sink.commands == [.setBackgroundPolling(enabled: false)])
    }

}
