//
//  SensorModelTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import Foundation
import OpenATKBindings
import OpenATKModel
import Testing

@MainActor
struct SensorModelTests {

    private let sink = RecordingSink()
    private let model: MouseModel

    init() {
        model = MouseModel(sink: sink)
    }

    private func connect(_ change: (inout SnapshotValue) -> Void = { _ in }) {
        var snapshot = SnapshotValue.sample()
        change(&snapshot)
        model.apply(.snapshot(snapshot: snapshot))
    }

    // MARK: Picks

    @Test func pickShownOnlyAfterConfirm() {
        connect()

        model.selectLiftOff(tenths: 12)

        #expect(sink.commands == [.setLiftOff(tenths: 12)])
        #expect(model.isPending)
        #expect(model.snapshot?.liftOffTenths == 7)

        connect { $0.liftOffTenths = 12 }

        #expect(model.snapshot?.liftOffTenths == 12)
    }

    @Test func failedPickKeepsTheRealValueAndRedraws() {
        connect()
        model.selectDebounce(ms: 4)
        let refreshCount = model.refreshCount

        model.apply(.commandFailed(message: "timeout"))

        #expect(model.snapshot?.debounceMs == 0)
        #expect(model.refreshCount == refreshCount + 1)
    }

    @Test func picksSendTheirCommands() {
        let picks: [(MouseModel) -> Void] = [
            { $0.selectDebounce(ms: 4) },
            { $0.selectMotionSync(enabled: true) },
            { $0.selectLinearCorrection(enabled: true) },
            { $0.selectRippleControl(enabled: true) },
            { $0.selectFarDistance(enabled: true) },
            { $0.selectAngleEnabled(true) },
            { $0.selectSleep(seconds: 60) }
        ]
        let expected: [Command] = [
            .setDebounce(ms: 4),
            .setMotionSync(enabled: true),
            .setLinearCorrection(enabled: true),
            .setRippleControl(enabled: true),
            .setFarDistance(enabled: true),
            .setAngleEnabled(enabled: true),
            .setSleep(seconds: 60)
        ]

        for (pick, command) in zip(picks, expected) {
            let sink = RecordingSink()
            let model = MouseModel(sink: sink)
            model.apply(.snapshot(snapshot: .sample()))

            pick(model)

            #expect(sink.commands == [command])
        }
    }

    @Test func pickingTheCurrentValueSendsNothing() {
        connect()

        model.selectLiftOff(tenths: 7)
        model.selectDebounce(ms: 0)
        model.selectMotionSync(enabled: false)
        model.selectLinearCorrection(enabled: false)
        model.selectRippleControl(enabled: false)
        model.selectFarDistance(enabled: false)
        model.selectAngleEnabled(false)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func picksOnAnUnknownFieldSendNothing() {
        connect {
            $0.liftOffTenths = nil
            $0.debounceMs = nil
            $0.motionSync = nil
            $0.linearCorrection = nil
            $0.rippleControl = nil
            $0.farDistance = nil
            $0.sleepSeconds = nil
        }

        model.selectLiftOff(tenths: 12)
        model.selectDebounce(ms: 4)
        model.selectMotionSync(enabled: true)
        model.selectLinearCorrection(enabled: true)
        model.selectRippleControl(enabled: true)
        model.selectFarDistance(enabled: true)
        model.selectSleep(seconds: 60)

        #expect(sink.commands.isEmpty)
    }

    @Test func picksWithoutAMouseSendNothing() {
        model.selectLiftOff(tenths: 12)
        model.selectSleep(seconds: 60)

        #expect(sink.commands.isEmpty)
    }

    // MARK: Off-list values

    @Test func offListValueIsDisabled() {
        connect()

        let choices = model.sleepChoices
        let listed = choices.dropLast().filter(\.isSelectable)

        #expect(choices.last == PickerChoice(value: 10, isSelectable: false))
        #expect(listed.count == 8)
        #expect(choices.count == 9)
    }

    @Test func listedValueAddsNoEntry() {
        connect { $0.sleepSeconds = 60 }

        #expect(model.sleepChoices.map(\.value) == [30, 60, 120, 180, 300, 1200, 1500, 1800])
    }

    @Test func offListValuesCannotBeWritten() {
        connect()

        model.selectSleep(seconds: 10)
        model.selectSleep(seconds: 45)
        model.selectDebounce(ms: 3)
        model.selectLiftOff(tenths: 18)

        #expect(sink.commands.isEmpty)
    }

    @Test func offListSleepIsNotRewrittenAfterAFailure() {
        connect()
        model.apply(.commandFailed(message: "timeout"))

        model.selectSleep(seconds: 10)

        #expect(sink.commands.isEmpty)
    }

    @Test func liftOffChoicesFollowTheSnapshotRange() {
        connect()

        #expect(model.liftOffChoices.map(\.value) == Array(7...17))
    }

    @Test func debounceChoicesFollowTheSnapshot() {
        connect()

        #expect(model.debounceChoices.map(\.value) == [0, 1, 2, 4, 8, 15, 20])
    }

    // MARK: Sleep

    @Test func sameSleepSendsNothingWithoutAFailure() {
        connect { $0.sleepSeconds = 60 }

        model.selectSleep(seconds: 60)

        #expect(sink.commands.isEmpty)
    }

    @Test func sameSleepHealsAfterAFailure() {
        connect { $0.sleepSeconds = 60 }
        model.apply(.commandFailed(message: "timeout"))

        model.selectSleep(seconds: 60)

        #expect(sink.commands == [.setSleep(seconds: 60)])
        #expect(model.errorText == nil)
    }

    // MARK: Debounce warning

    @Test(arguments: [UInt8(0), 1, 2])
    func debounceWarningBelowFour(ms: UInt8) {
        connect { $0.debounceMs = ms }

        #expect(model.showsDebounceWarning)
    }

    @Test(arguments: [UInt8(4), 8, 20])
    func noDebounceWarningFromFour(ms: UInt8) {
        connect { $0.debounceMs = ms }

        #expect(!model.showsDebounceWarning)
    }

    @Test func noDebounceWarningWhenUnknown() {
        connect { $0.debounceMs = nil }

        #expect(!model.showsDebounceWarning)
    }

    // MARK: Angle

    @Test func angleHiddenWhenAbsent() {
        connect { $0.angle = nil }

        model.editField(.angle, text: "5")
        model.commitField(.angle)
        model.selectAngleEnabled(true)

        #expect(!model.showsAngle)
        #expect(sink.commands.isEmpty)
    }

    @Test func angleShownWhenPresent() {
        connect()

        #expect(model.showsAngle)
        #expect(model.fieldText(.angle) == "0")
    }

    @Test(arguments: [Int8(-30), -1, 1, 30])
    func angleCommitsInsideTheRange(degrees: Int8) {
        connect()
        model.editField(.angle, text: String(degrees))

        model.commitField(.angle)

        #expect(sink.commands == [.setAngle(degrees: degrees)])
    }

    @Test(arguments: ["31", "-31", "128", "", "-", "abc", "1.5"])
    func angleRefusesOtherText(text: String) {
        connect()
        model.editField(.angle, text: text)

        model.commitField(.angle)

        #expect(sink.commands.isEmpty)
        #expect(model.fieldText(.angle) == "0")
    }

    @Test func angleEqualToTheShownValueSendsNothing() {
        connect { $0.angle = AngleValue(degrees: 5, enabled: true) }
        model.editField(.angle, text: "05")

        model.commitField(.angle)

        #expect(sink.commands.isEmpty)
    }

    @Test func unknownAngleDegreesCannotCommit() {
        connect { $0.angle = AngleValue(degrees: nil, enabled: true) }
        model.editField(.angle, text: "5")

        model.commitField(.angle)

        #expect(sink.commands.isEmpty)
    }

    @Test func unknownAngleSwitchCannotChange() {
        connect { $0.angle = AngleValue(degrees: 5, enabled: nil) }

        model.selectAngleEnabled(true)

        #expect(sink.commands.isEmpty)
    }

    @Test func committedAngleStaysUntilTheMouseAnswers() {
        connect()
        model.editField(.angle, text: "12")

        model.commitField(.angle)
        model.commitField(.angle)
        model.discardFieldEdit()

        #expect(model.fieldText(.angle) == "12")
        #expect(sink.commands == [.setAngle(degrees: 12)])

        connect { $0.angle = AngleValue(degrees: 12, enabled: false) }

        #expect(model.fieldText(.angle) == "12")
    }

    @Test func angleDraftAndStageDraftShareOneSlot() {
        connect()
        model.editField(.angle, text: "12")

        model.editField(.stage(1), text: "900")

        #expect(sink.commands == [.setAngle(degrees: 12)])
        #expect(model.fieldText(.stage(1)) == "800")
    }

}
