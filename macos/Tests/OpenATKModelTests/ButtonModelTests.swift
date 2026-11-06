//
//  ButtonModelTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

@MainActor
struct ButtonModelTests {

    private let sink = RecordingSink()
    private let model: MouseModel

    init() {
        model = MouseModel(sink: sink)
        model.apply(.snapshot(snapshot: .sample(buttons: [
            ButtonValue(button: .left, action: .mouseLeft, keys: nil),
            ButtonValue(
                button: .side1,
                action: .shortcut,
                keys: [.modifier(mask: 1), .general(usage: 0x06)]
            ),
            ButtonValue(button: .side2, action: .shortcut, keys: [.general(usage: 0x4d)]),
            ButtonValue(button: .middle, action: .shortcut, keys: nil),
            ButtonValue(button: .right, action: .other(class: 9, value1: 1, value2: 2), keys: nil),
        ])))
    }

    @Test func selectActionSendsSetButton() {
        model.selectAction(button: .side1, action: .dpiCycle)

        #expect(sink.commands == [.setButton(button: .side1, action: .dpiCycle)])
        #expect(model.isPending)
    }

    @Test func selectActionToCurrentSendsNothing() {
        model.selectAction(button: .left, action: .mouseLeft)

        #expect(sink.commands.isEmpty)
    }

    @Test func pickerShowsRealActionWhilePending() {
        model.selectAction(button: .left, action: .mouseRight)

        #expect(model.isPending)
        #expect(model.snapshot?.buttons.first?.action == .mouseLeft)
    }

    @Test func pickDuringPendingSendsNothing() {
        model.selectAction(button: .left, action: .mouseRight)

        model.selectAction(button: .side1, action: .dpiCycle)
        model.setShortcut(button: .side2, keys: [.general(usage: 0x06)])

        #expect(sink.commands == [.setButton(button: .left, action: .mouseRight)])
    }

    @Test func setShortcutSendsKeys() {
        let keys: [ShortcutKeyValue] = [.modifier(mask: 1), .general(usage: 0x06)]

        model.setShortcut(button: .left, keys: keys)

        #expect(sink.commands == [.setShortcut(button: .left, keys: keys)])
    }

    @Test func threeModifiersAreRefused() {
        let keys: [ShortcutKeyValue] = [
            .modifier(mask: 1), .modifier(mask: 2), .modifier(mask: 4), .general(usage: 0x06),
        ]

        model.setShortcut(button: .left, keys: keys)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func shortcutWithoutAKeyIsRefused() {
        model.setShortcut(button: .left, keys: [.modifier(mask: 1)])

        #expect(sink.commands.isEmpty)
    }

    @Test func shortcutTextForCtrlC() {
        #expect(model.shortcutText(button: .side1) == "Ctrl+C")
    }

    @Test func shortcutTextForSingleEnd() {
        #expect(model.shortcutText(button: .side2) == "End")
    }

    @Test func unknownShapeShowsLabel() {
        #expect(model.shortcutText(button: .middle) == Strings.unknownShortcut)
    }

    @Test func otherActionHasNoPicker() {
        #expect(!model.isEditable(button: .right))
        #expect(model.isEditable(button: .left))
    }

    @Test func captureTakesAKeyAndCloses() {
        model.beginCapture(button: .left)

        let keys = model.captureKey(code: 8, modifiers: [.control], isRepeat: false)

        #expect(keys == [.modifier(mask: 1), .general(usage: 0x06)])
        #expect(model.captureButton == nil)
    }

    @Test func unmappedKeyShowsMessageAndStaysInCapture() {
        model.beginCapture(button: .left)

        let keys = model.captureKey(code: 0xFFFF, modifiers: [], isRepeat: false)

        #expect(keys == nil)
        #expect(model.captureButton == .left)
        #expect(model.captureMessage == Strings.unmappedKey)
    }

    @Test func threeModifiersInCaptureShowMessageAndStay() {
        model.beginCapture(button: .left)

        let keys = model.captureKey(
            code: 8,
            modifiers: [.control, .shift, .option],
            isRepeat: false
        )

        #expect(keys == nil)
        #expect(model.captureButton == .left)
        #expect(model.captureMessage == Strings.tooManyModifiers)
    }

    @Test func cancelClosesCapture() {
        model.beginCapture(button: .left)

        model.cancelCapture()

        #expect(model.captureButton == nil)
        #expect(model.captureMessage == nil)
    }

    @Test func capturingStoredShortcutSendsNothing() {
        model.setShortcut(button: .side1, keys: [.modifier(mask: 1), .general(usage: 0x06)])

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func autoRepeatIsIgnored() {
        model.beginCapture(button: .left)

        let keys = model.captureKey(code: 8, modifiers: [], isRepeat: true)

        #expect(keys == nil)
        #expect(model.captureButton == .left)
        #expect(model.captureMessage == nil)
    }

    @Test func beginCaptureIsRefusedWhilePending() {
        model.selectAction(button: .left, action: .mouseRight)

        model.beginCapture(button: .side1)

        #expect(model.captureButton == nil)
    }

    @Test func writeClosesCapture() {
        model.beginCapture(button: .left)

        model.selectAction(button: .side1, action: .dpiCycle)

        #expect(model.captureButton == nil)
    }

    @Test func notConnectedClosesCapture() {
        model.beginCapture(button: .left)

        model.apply(.notConnected)

        #expect(model.captureButton == nil)
    }

    // MARK: Requests

    @Test func losingTheLastLeftClickNeedsConfirmation() {
        let needsConfirmation = model.request(button: .left, change: .action(.mouseRight))

        #expect(needsConfirmation)
        #expect(model.confirmation == ButtonRequest(button: .left, change: .action(.mouseRight)))
        #expect(sink.commands.isEmpty)
    }

    @Test func confirmingSendsTheRequest() throws {
        model.request(button: .left, change: .action(.mouseRight))
        let request = try #require(model.confirmation)

        model.confirm(request)

        #expect(sink.commands == [.setButton(button: .left, action: .mouseRight)])
        #expect(model.confirmation == nil)
    }

    @Test func cancellingSendsNothing() {
        model.request(button: .left, change: .action(.mouseRight))

        model.cancelConfirmation()

        #expect(model.confirmation == nil)
        #expect(sink.commands.isEmpty)
    }

    @Test func harmlessRequestSendsAtOnce() {
        let needsConfirmation = model.request(button: .side1, change: .action(.dpiCycle))

        #expect(!needsConfirmation)
        #expect(sink.commands == [.setButton(button: .side1, action: .dpiCycle)])
    }

    @Test func shortcutOnTheLastLeftClickNeedsConfirmation() {
        let keys: [ShortcutKeyValue] = [.general(usage: 0x06)]

        let needsConfirmation = model.request(button: .left, change: .shortcut(keys))

        #expect(needsConfirmation)
        #expect(sink.commands.isEmpty)
    }

    @Test func requestClosesCapture() {
        model.beginCapture(button: .left)

        model.request(button: .side1, change: .action(.dpiCycle))

        #expect(model.captureButton == nil)
    }

    @Test func notConnectedDropsConfirmation() {
        model.request(button: .left, change: .action(.mouseRight))

        model.apply(.notConnected)

        #expect(model.confirmation == nil)
    }

    @Test func refreshCountGrowsOnSnapshotAndFailure() {
        let before = model.refreshCount

        model.apply(.snapshot(snapshot: .sample()))
        model.apply(.commandFailed(message: "timeout"))
        model.apply(.battery(percent: 50, charging: false))

        #expect(model.refreshCount == before + 2)
    }

}
