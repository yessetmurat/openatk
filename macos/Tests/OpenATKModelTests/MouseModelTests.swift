//
//  MouseModelTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import Dispatch
import OpenATKBindings
import OpenATKModel
import Testing

@MainActor
struct MouseModelTests {

    private let sink = RecordingSink()
    private let model: MouseModel

    init() {
        model = MouseModel(sink: sink)
    }

    // MARK: Reducer

    @Test func startsNotConnected() {
        #expect(model.snapshot == nil)
        #expect(!model.isPending)
        #expect(model.errorText == nil)
        #expect(sink.commands.isEmpty)
    }

    @Test func snapshotClearsPending() {
        model.apply(.snapshot(snapshot: .sample()))
        model.selectPolling(hz: 500)

        model.apply(.snapshot(snapshot: .sample(pollingHz: 500)))

        #expect(!model.isPending)
        #expect(model.snapshot?.pollingHz == 500)
    }

    @Test func failureKeepsRealValueAndSetsError() {
        model.apply(.snapshot(snapshot: .sample()))
        model.selectPolling(hz: 500)

        model.apply(.commandFailed(message: "timeout"))

        #expect(!model.isPending)
        #expect(model.errorText == "timeout")
        #expect(model.snapshot?.pollingHz == 250)
    }

    @Test func errorStaysThroughUpdatesAndNonWrites() {
        model.apply(.snapshot(snapshot: .sample()))
        model.apply(.commandFailed(message: "timeout"))

        model.apply(.snapshot(snapshot: .sample()))
        model.apply(.battery(percent: 50, charging: true))
        model.apply(.notConnected)
        model.apply(.snapshot(snapshot: .sample()))
        model.send(.refresh)
        model.setBackgroundPolling(true)

        #expect(model.errorText == "timeout")
    }

    @Test func errorStaysUntilNextSend() {
        model.apply(.snapshot(snapshot: .sample()))
        model.apply(.commandFailed(message: "timeout"))

        model.selectPolling(hz: 500)

        #expect(model.errorText == nil)
    }

    @Test func notConnectedClearsSnapshotAndPending() {
        model.apply(.snapshot(snapshot: .sample()))
        model.selectPolling(hz: 500)

        model.apply(.notConnected)

        #expect(model.snapshot == nil)
        #expect(!model.isPending)
    }

    @Test func batteryWithoutSnapshotIsIgnored() {
        model.apply(.battery(percent: 10, charging: false))

        #expect(model.snapshot == nil)
    }

    @Test func batteryUpdatesSnapshot() {
        model.apply(.snapshot(snapshot: .sample()))

        model.apply(.battery(percent: 50, charging: true))

        #expect(model.snapshot?.percent == 50)
        #expect(model.snapshot?.charging == true)
    }

    // MARK: Sending

    @Test func sendRefusedWhilePending() {
        model.apply(.snapshot(snapshot: .sample()))

        model.send(.setPolling(hz: 500))
        model.send(.setPolling(hz: 1000))
        model.send(.refresh)

        #expect(sink.commands == [.setPolling(hz: 500)])
    }

    @Test func backgroundPollingNeverPending() {
        model.apply(.snapshot(snapshot: .sample()))

        model.setBackgroundPolling(true)

        #expect(!model.isPending)
    }

    @Test func backgroundPollingGoesOutWhilePending() {
        model.apply(.snapshot(snapshot: .sample()))
        model.selectPolling(hz: 500)

        model.setBackgroundPolling(false)

        #expect(sink.commands == [.setPolling(hz: 500), .setBackgroundPolling(enabled: false)])
    }

    @Test func refreshNeverPending() {
        model.send(.refresh)

        #expect(!model.isPending)
        #expect(sink.commands == [.refresh])
    }

    @Test func pickerShowsRealRateWhilePending() {
        model.apply(.snapshot(snapshot: .sample()))

        model.selectPolling(hz: 500)

        #expect(model.isPending)
        #expect(model.snapshot?.pollingHz == 250)
        #expect(sink.commands == [.setPolling(hz: 500)])
    }

    @Test func pickingTheCurrentValueSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))

        model.selectPolling(hz: 250)
        model.selectStage(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func selectStageSendsIndex() {
        model.apply(.snapshot(snapshot: .sample()))

        model.selectStage(index: 3)

        #expect(sink.commands == [.setCurrentStage(index: 3)])
    }

    @Test(arguments: [(1, -1), (8, 1)])
    func stageCountStaysBetweenOneAndEight(count: Int, delta: Int) {
        model.apply(.snapshot(snapshot: .sample(stages: Array(repeating: 800, count: count))))

        model.changeStageCount(by: delta)

        #expect(sink.commands.isEmpty)
    }

    @Test func stageCountChangesByOne() {
        model.apply(.snapshot(snapshot: .sample(stages: [800])))

        model.changeStageCount(by: 1)

        #expect(sink.commands == [.setStageCount(count: 2)])
    }

    // MARK: DPI

    @Test func stageTextShowsTheMouseValue() {
        model.apply(.snapshot(snapshot: .sample()))

        #expect(model.stageText(index: 1) == "800")
    }

    @Test func stageTextShowsTheDraftOnlyOnItsStage() {
        model.apply(.snapshot(snapshot: .sample()))

        model.editStage(index: 1, text: "90")

        #expect(model.stageText(index: 1) == "90")
        #expect(model.stageText(index: 2) == "1200")
    }

    @Test func commitWithoutEditSendsNothing() {
        model.apply(.snapshot(snapshot: .sample(stages: [400, 12345])))

        model.commitStage(index: 0)
        model.commitStage(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func commitOfUnchangedTextSendsNothing() {
        model.apply(.snapshot(snapshot: .sample(stages: [400, 12345])))

        model.editStage(index: 1, text: "12345")
        model.commitStage(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func commitSnapsToGrid() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "1234")

        model.commitStage(index: 1)

        #expect(sink.commands == [.setStageDpi(index: 1, x: 1230, y: 1230)])
    }

    @Test func commitShowsTheMouseValueUntilConfirmed() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "1234")

        model.commitStage(index: 1)

        #expect(model.stageText(index: 1) == "800")
    }

    @Test func commitOfAnotherStageSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "1234")

        model.commitStage(index: 2)

        #expect(sink.commands.isEmpty)
    }

    @Test func commitSnappingToShownValueSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "804")

        model.commitStage(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
        #expect(model.stageText(index: 1) == "800")
    }

    @Test func nonNumericTextSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))

        for text in ["", "abc", "12a", "-5", "1.5", "99999999999"] {
            model.editStage(index: 0, text: text)
            model.commitStage(index: 0)
        }

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
        #expect(model.stageText(index: 0) == "400")
    }

    @Test func stageWithSplitAxesCannotCommit() {
        var snapshot = SnapshotValue.sample()
        snapshot.stages[0] = StageValue(x: 400, y: 800)
        model.apply(.snapshot(snapshot: snapshot))
        model.editStage(index: 0, text: "1000")

        model.commitStage(index: 0)

        #expect(sink.commands.isEmpty)
    }

    @Test func commitOutsideStagesSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 4, text: "800")

        model.commitStage(index: 4)

        #expect(sink.commands.isEmpty)
    }

    @Test func commitWithoutMouseSendsNothing() {
        model.editStage(index: 0, text: "800")

        model.commitStage(index: 0)

        #expect(sink.commands.isEmpty)
    }

    // MARK: DPI draft

    @Test func draftClearedByFailedWrite() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.apply(.commandFailed(message: "timeout"))
        model.commitStage(index: 0)

        #expect(model.stageText(index: 0) == "400")
        #expect(sink.commands.isEmpty)
    }

    @Test func draftClearedByIdenticalSnapshot() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.apply(.snapshot(snapshot: .sample()))
        model.commitStage(index: 0)

        #expect(model.stageText(index: 0) == "400")
        #expect(sink.commands.isEmpty)
    }

    @Test func draftClearedByNotConnected() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.apply(.notConnected)
        model.apply(.battery(percent: 50, charging: false))
        model.apply(.snapshot(snapshot: .sample()))

        #expect(model.stageText(index: 0) == "400")
    }

    @Test func draftKeptByBattery() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.apply(.battery(percent: 50, charging: false))
        model.commitStage(index: 0)

        #expect(sink.commands == [.setStageDpi(index: 0, x: 900, y: 900)])
    }

    @Test func draftClearedByOtherWrite() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.selectPolling(hz: 500)
        model.apply(.snapshot(snapshot: .sample()))
        model.commitStage(index: 0)

        #expect(model.stageText(index: 0) == "400")
        #expect(sink.commands == [.setPolling(hz: 500)])
    }

    @Test func draftKeptByRefreshUntilItsSnapshot() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.send(.refresh)

        #expect(model.stageText(index: 0) == "900")
    }

    @Test func editIgnoredWhilePending() {
        model.apply(.snapshot(snapshot: .sample()))
        model.selectPolling(hz: 500)

        model.editStage(index: 0, text: "900")
        model.apply(.snapshot(snapshot: .sample()))
        model.commitStage(index: 0)

        #expect(model.stageText(index: 0) == "400")
        #expect(sink.commands == [.setPolling(hz: 500)])
    }

    @Test func echoFromAnotherFieldKeepsDraft() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "900")

        model.editStage(index: 2, text: "1200")
        model.commitStage(index: 1)

        #expect(sink.commands == [.setStageDpi(index: 1, x: 900, y: 900)])
    }

    @Test func editInAnotherFieldCommitsTheLiveDraft() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "900")

        model.editStage(index: 2, text: "1500")

        #expect(sink.commands == [.setStageDpi(index: 1, x: 900, y: 900)])
    }

    @Test func editThatCommitsAWriteIsDropped() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "900")

        model.editStage(index: 2, text: "1500")
        model.apply(.snapshot(snapshot: .sample()))
        model.commitStage(index: 2)

        #expect(model.stageText(index: 2) == "1200")
        #expect(sink.commands == [.setStageDpi(index: 1, x: 900, y: 900)])
    }

    @Test func editInAnotherFieldIsKeptWhenNothingWasWritten() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "abc")

        model.editStage(index: 2, text: "1500")

        #expect(model.stageText(index: 1) == "800")
        #expect(model.stageText(index: 2) == "1500")
        #expect(sink.commands.isEmpty)
    }

    @Test func retypingTheShownValueSendsNothing() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 1, text: "900")

        model.editStage(index: 1, text: "800")
        model.commitStage(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(model.stageText(index: 1) == "800")
    }

    @Test func escapeDiscardsDraft() {
        model.apply(.snapshot(snapshot: .sample()))
        model.editStage(index: 0, text: "900")

        model.discardStageEdit()
        model.commitStage(index: 0)

        #expect(model.stageText(index: 0) == "400")
        #expect(sink.commands.isEmpty)
    }

    // MARK: Listener

    @Test func updateAfterWindowClosedStillApplies() async {
        let listener = MouseUpdateListener()
        listener.model = model

        await deliver([.snapshot(snapshot: .sample(pollingHz: 500))], to: listener)

        #expect(model.snapshot?.pollingHz == 500)
    }

    /// Calls the listener off the main thread, as the service does, and waits for the hops:
    /// the main queue is FIFO, so a block queued after them runs after them.
    private func deliver(_ updates: [MouseUpdate], to listener: MouseUpdateListener) async {
        await Task.detached {
            for update in updates {
                listener.onEvent(event: update)
            }
        }.value

        await withCheckedContinuation { continuation in
            DispatchQueue.main.async {
                continuation.resume()
            }
        }
    }

}
