//
//  LaunchAtLoginSettingTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import ServiceManagement
import Testing

@MainActor
struct LaunchAtLoginSettingTests {

    private let service = FakeLaunchAtLogin()

    private func makeSetting() -> LaunchAtLoginSetting {
        LaunchAtLoginSetting(service: service)
    }

    @Test func offByDefault() {
        let setting = makeSetting()

        #expect(!setting.isEnabled)
        #expect(service.calls.isEmpty)
    }

    @Test func turningOnRegistersAndReadsTheStatus() {
        let setting = makeSetting()

        setting.set(enabled: true)

        #expect(service.calls == ["register"])
        #expect(setting.isEnabled)
    }

    @Test func turningOffUnregisters() {
        service.status = .enabled
        let setting = makeSetting()

        setting.set(enabled: false)

        #expect(service.calls == ["unregister"])
        #expect(!setting.isEnabled)
    }

    @Test func settingTheShownStateCallsNothing() {
        let setting = makeSetting()

        setting.set(enabled: false)

        #expect(service.calls.isEmpty)
    }

    @Test func launchFailureKeepsRealState() {
        service.fails = true
        let setting = makeSetting()

        setting.set(enabled: true)

        #expect(!setting.isEnabled)
        #expect(setting.errorText != nil)
        #expect(setting.refreshCount == 1)
    }

    @Test func failedTurnOffStaysOn() {
        service.status = .enabled
        service.fails = true
        let setting = makeSetting()

        setting.set(enabled: false)

        #expect(setting.isEnabled)
        #expect(setting.errorText != nil)
    }

    @Test func successfulChangeClearsTheError() {
        service.fails = true
        let setting = makeSetting()
        setting.set(enabled: true)
        service.fails = false

        setting.set(enabled: true)

        #expect(setting.isEnabled)
        #expect(setting.errorText == nil)
    }

    @Test func settingTheShownStateKeepsTheError() {
        service.fails = true
        let setting = makeSetting()
        setting.set(enabled: true)

        setting.set(enabled: false)

        #expect(setting.errorText != nil)
    }

    @Test func requiresApprovalIsNotOnAndAsksForApproval() {
        service.statusAfterRegister = .requiresApproval
        let setting = makeSetting()

        setting.set(enabled: true)

        #expect(!setting.isEnabled)
        #expect(setting.needsApproval)
        #expect(setting.errorText == nil)
    }

    @Test func registerThatChangesNothingStillRedraws() {
        service.status = .requiresApproval
        service.statusAfterRegister = .requiresApproval
        let setting = makeSetting()

        setting.set(enabled: true)

        #expect(!setting.isEnabled)
        #expect(setting.refreshCount == 1)
    }

    @Test func refreshPicksUpAChangeMadeInSystemSettings() {
        let setting = makeSetting()
        service.status = .enabled

        setting.refresh()

        #expect(setting.isEnabled)
    }

}
