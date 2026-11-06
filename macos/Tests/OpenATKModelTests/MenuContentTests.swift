//
//  MenuContentTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

@MainActor
struct MenuContentTests {

    private static func entries(_ snapshot: SnapshotValue?) -> [MenuEntry] {
        MenuContent.entries(snapshot: snapshot, strings: Strings.self)
    }

    private static func choices(_ snapshot: SnapshotValue?) -> [MenuChoice] {
        entries(snapshot).compactMap { entry in
            if case .choice(let choice) = entry {
                return choice
            }

            return nil
        }
    }

    private static func stage(_ index: Int, _ title: String, isChecked: Bool = false) -> MenuEntry {
        .choice(MenuChoice(title: title, target: .stage(index: index), isChecked: isChecked))
    }

    private static func rate(_ hertz: UInt32, isChecked: Bool = false) -> MenuEntry {
        .choice(MenuChoice(title: "\(hertz) Hz", target: .rate(hertz: hertz), isChecked: isChecked))
    }

    @Test func menuListsBatteryStagesRatesOpenQuitInOrder() {
        let entries = Self.entries(.sample())

        #expect(entries == [
            .info("Battery: 87%"),
            .separator,
            Self.stage(0, "400 DPI"),
            Self.stage(1, "800 DPI", isChecked: true),
            Self.stage(2, "1200 DPI"),
            Self.stage(3, "1600 DPI"),
            .separator,
            Self.rate(125),
            Self.rate(250, isChecked: true),
            Self.rate(500),
            Self.rate(1000),
            .separator,
            .openWindow("Open full window"),
            .quit("Quit openatk"),
        ])
    }

    @Test func currentStageAndRateAreChecked() {
        let checked = Self.choices(.sample(pollingHz: 1000)).filter(\.isChecked).map(\.target)

        #expect(checked == [.stage(index: 1), .rate(hertz: 1000)])
    }

    @Test func splitStageShowsBothAxes() {
        var snapshot = SnapshotValue.sample()
        snapshot.stages[0] = StageValue(x: 400, y: 800)

        let title = Self.choices(snapshot).first?.title

        #expect(title == "X 400, Y 800")
    }

    @Test func rateOutsideOptionsIsDisabled() {
        let rates = Self.choices(.sample(pollingHz: 8000)).filter { choice in
            if case .rate = choice.target {
                return true
            }

            return false
        }

        #expect(rates.map(\.title) == ["125 Hz", "250 Hz", "500 Hz", "1000 Hz", "8000 Hz"])
        #expect(rates.map(\.isEnabled) == [true, true, true, true, false])
        #expect(rates.filter(\.isChecked).map(\.title) == ["8000 Hz"])
    }

    @Test func rateInsideOptionsIsNotAddedTwice() {
        let rates = Self.choices(.sample(pollingHz: 500)).filter { $0.title == "500 Hz" }

        #expect(rates.count == 1)
        #expect(rates.first?.isEnabled == true)
    }

    @Test func notConnectedMenuHasOnlyOpenAndQuit() {
        let entries = Self.entries(nil)

        #expect(entries == [.openWindow("Open full window"), .quit("Quit openatk")])
    }

    @Test func titleIsPercent() {
        #expect(MenuContent.title(snapshot: .sample()) == "87%")
    }

    @Test func titleWhenNotConnected() {
        #expect(MenuContent.title(snapshot: nil) == "--")
    }

    @Test func menuToggleCallsTheModelOnly() throws {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        model.apply(.snapshot(snapshot: .sample()))
        let choices = Self.choices(model.snapshot)
        let stage = try #require(choices.first { $0.target == .stage(index: 3) })
        let rate = try #require(choices.first { $0.target == .rate(hertz: 500) })

        model.pick(stage)
        model.pick(rate)

        #expect(sink.commands == [.setCurrentStage(index: 3)])
        #expect(Self.choices(model.snapshot) == choices)
    }

    @Test func pickingTheCheckedChoiceSendsNothing() {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        model.apply(.snapshot(snapshot: .sample()))
        let checked = Self.choices(model.snapshot).filter(\.isChecked)

        checked.forEach(model.pick)

        #expect(sink.commands.isEmpty)
    }

    @Test func pickedRateIsSent() throws {
        let sink = RecordingSink()
        let model = MouseModel(sink: sink)
        model.apply(.snapshot(snapshot: .sample()))
        let choices = Self.choices(model.snapshot)
        let rate = try #require(choices.first { $0.target == .rate(hertz: 500) })

        model.pick(rate)

        #expect(sink.commands == [.setPolling(hz: 500)])
    }

}
