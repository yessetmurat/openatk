//
//  MouseModel.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import Observation
import OpenATKBindings

/// The app's state: only what the mouse reported, plus whether a write is in flight.
@MainActor
@Observable
public final class MouseModel {

    /// The numbers of DPI stages the mouse accepts.
    public static let stageCountRange = 1...8

    /// The settings as read from the mouse, or `nil` when no mouse can be reached.
    public private(set) var snapshot: SnapshotValue?

    /// Whether a write was sent and its result has not arrived.
    public private(set) var isPending = false

    /// Why the last write or background operation failed.
    public private(set) var errorText: String?

    /// A change held until the user confirms it.
    public internal(set) var confirmation: ButtonRequest?

    /// Counts the snapshots and failures received. A view that applies it as its identity is
    /// rebuilt on each, so a control the user changed without effect shows the mouse's value.
    public private(set) var refreshCount = 0

    /// The button whose shortcut key is being captured.
    public internal(set) var captureButton: MouseButton?

    /// Why the last captured key was refused.
    public internal(set) var captureMessage: String?

    /// Text typed into a numeric field and not yet written. There is only one: an edit in another
    /// field commits this one first, so no typed text is replaced unseen.
    private var draft: FieldDraft?

    @ObservationIgnored private let sink: any CommandSink

    /// Creates a model with no mouse, which sends through `sink`.
    public init(sink: any CommandSink) {
        self.sink = sink
    }

    /// Applies an update from the mouse.
    public func apply(_ update: MouseUpdate) {
        switch update {
        case .notConnected:
            snapshot = nil
            isPending = false
            draft = nil
            cancelCapture()
            confirmation = nil
        case .snapshot(let snapshot):
            refreshCount += 1
            self.snapshot = snapshot
            isPending = false
            draft = nil
        case .battery(let percent, let charging):
            snapshot?.percent = percent
            snapshot?.charging = charging
        case .commandFailed(let message):
            refreshCount += 1
            errorText = message
            isPending = false
            draft = nil
        }
    }

    /// Sends `command`, unless a write is in flight. A write clears the error and any typed text,
    /// and stays pending until its result arrives.
    public func send(_ command: Command) {
        guard !isPending else {
            return
        }

        if command.isWrite {
            errorText = nil
            isPending = true
            draft = nil
            cancelCapture()
            confirmation = nil
        }

        sink.send(command: command)
    }

    /// Turns background polling on or off. No event follows it, so it never marks a write
    /// pending, and it goes out even while one is.
    public func setBackgroundPolling(_ enabled: Bool) {
        sink.send(command: .setBackgroundPolling(enabled: enabled))
    }

    /// The text of `field`: what is being typed there, else the mouse's value.
    public func fieldText(_ field: FieldDraft.Field) -> String {
        if let draft,
           draft.field == field {
            return draft.text
        }

        switch field {
        case .stage(let index):
            guard let snapshot,
                  snapshot.stages.indices.contains(index) else {
                return ""
            }

            return String(snapshot.stages[index].x)
        case .angle:
            return snapshot?.angle?.degrees.map { String($0) } ?? ""
        }
    }

    /// Records text typed into `field`.
    ///
    /// Ignored while a write is in flight, so a late call from the field cannot bring back text
    /// that was dropped. Text the field already shows is not an edit: a field reports it when it
    /// gains focus, which must not replace the draft of the field that is losing focus.
    ///
    /// A real edit in another field commits the live draft first. If that starts a write, this
    /// edit is dropped like any other edit while a write is in flight.
    public func editField(_ field: FieldDraft.Field, text: String) {
        guard !isPending,
              text != fieldText(field) else {
            return
        }

        if let draft,
           draft.field != field {
            commitField(draft.field)
        }

        guard !isPending else {
            return
        }

        draft = FieldDraft(field: field, text: text, isSent: false)
    }

    /// Drops the typed text, so the field shows the mouse's value again. A draft whose write is
    /// in flight stays until the mouse answers.
    public func discardFieldEdit() {
        guard draft?.isSent != true else {
            return
        }

        draft = nil
    }

    /// Writes the text typed for `field`. The text stays shown until the mouse answers a write;
    /// without a write it is dropped.
    ///
    /// Sends nothing when nothing was typed for this field, or when the text does not make a
    /// write: see ``angleCommand(text:)`` and ``stageCommand(index:text:)``.
    public func commitField(_ field: FieldDraft.Field) {
        guard let draft,
              draft.field == field,
              !draft.isSent else {
            return
        }

        discardFieldEdit()

        guard let command = command(for: draft) else {
            return
        }

        send(command)

        if isPending {
            self.draft = FieldDraft(field: field, text: draft.text, isSent: true)
        }
    }

    /// Sets the polling rate, unless it is the current one.
    public func selectPolling(hz: UInt32) {
        guard let snapshot,
              hz != snapshot.pollingHz else {
            return
        }

        send(.setPolling(hz: hz))
    }

    /// Makes the stage at `index` the current one, unless it already is.
    public func selectStage(index: Int) {
        guard let snapshot,
              snapshot.stages.indices.contains(index),
              index != Int(snapshot.currentStage) else {
            return
        }

        send(.setCurrentStage(index: UInt8(index)))
    }

    /// Adds `delta` to the number of stages, within ``stageCountRange``.
    public func changeStageCount(by delta: Int) {
        guard let snapshot else {
            return
        }

        let count = snapshot.stages.count + delta

        guard Self.stageCountRange.contains(count) else {
            return
        }

        send(.setStageCount(count: UInt8(count)))
    }

    private func command(for draft: FieldDraft) -> Command? {
        switch draft.field {
        case .stage(let index):
            stageCommand(index: index, text: draft.text)
        case .angle:
            angleCommand(text: draft.text)
        }
    }

    /// The write for text typed for the stage at `index`, snapped to the sensor's grid, or `nil`
    /// when the text is the shown value, is not a number or snaps to the shown value, or when the
    /// stage has different X and Y values, which this app shows read-only.
    private func stageCommand(index: Int, text: String) -> Command? {
        guard let snapshot,
              snapshot.stages.indices.contains(index) else {
            return nil
        }

        let stage = snapshot.stages[index]

        guard stage.x == stage.y,
              text != String(stage.x),
              let typed = UInt32(text) else {
            return nil
        }

        let dpi = nearestValidDpi(snapshot: snapshot, value: typed)

        guard dpi != stage.x else {
            return nil
        }

        return .setStageDpi(index: UInt8(index), x: dpi, y: dpi)
    }

}

private extension Command {

    /// Whether the mouse answers this command with an event that ends the pending state.
    var isWrite: Bool {
        switch self {
        case .setPolling, .setCurrentStage, .setStageCount, .setStageDpi, .setButton, .setShortcut,
             .setLiftOff, .setDebounce, .setMotionSync, .setLinearCorrection, .setRippleControl,
             .setSleep, .setAngle, .setAngleEnabled, .setFarDistance:
            return true
        case .setBackgroundPolling, .refresh:
            return false
        }
    }

}
