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

    /// Text typed into a DPI field and not yet written. There is only one: an edit in another
    /// field commits this one first, so no typed text is replaced unseen.
    private var stageDraft: (index: Int, text: String)?

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
            stageDraft = nil
        case .snapshot(let snapshot):
            self.snapshot = snapshot
            isPending = false
            stageDraft = nil
        case .battery(let percent, let charging):
            snapshot?.percent = percent
            snapshot?.charging = charging
        case .commandFailed(let message):
            errorText = message
            isPending = false
            stageDraft = nil
        }
    }

    /// Sends `command`, unless a write is in flight. A write clears the error and any typed DPI,
    /// and stays pending until its result arrives.
    public func send(_ command: Command) {
        guard !isPending else {
            return
        }

        if command.isWrite {
            errorText = nil
            isPending = true
            stageDraft = nil
        }

        sink.send(command: command)
    }

    /// Turns background polling on or off. No event follows it, so it never marks a write
    /// pending, and it goes out even while one is.
    public func setBackgroundPolling(_ enabled: Bool) {
        sink.send(command: .setBackgroundPolling(enabled: enabled))
    }

    /// The text of the stage's DPI field: what is being typed there, else the mouse's value.
    public func stageText(index: Int) -> String {
        if let stageDraft,
           stageDraft.index == index {
            return stageDraft.text
        }

        guard let snapshot,
              snapshot.stages.indices.contains(index) else {
            return ""
        }

        return String(snapshot.stages[index].x)
    }

    /// Records text typed into the stage's DPI field.
    ///
    /// Ignored while a write is in flight, so a late call from the field cannot bring back text
    /// that was dropped. Text the field already shows is not an edit: a field reports it when it
    /// gains focus, which must not replace the draft of the field that is losing focus.
    ///
    /// A real edit in another field commits the live draft first. If that starts a write, this
    /// edit is dropped like any other edit while a write is in flight.
    public func editStage(index: Int, text: String) {
        guard !isPending,
              text != stageText(index: index) else {
            return
        }

        if let stageDraft,
           stageDraft.index != index {
            commitStage(index: stageDraft.index)
        }

        guard !isPending else {
            return
        }

        stageDraft = (index, text)
    }

    /// Drops the typed text, so the field shows the mouse's value again.
    public func discardStageEdit() {
        stageDraft = nil
    }

    /// Writes the text typed for the stage at `index`, snapped to the sensor's grid, and drops it.
    ///
    /// Sends nothing when nothing was typed for this stage, when the text is the shown value, is
    /// not a number or snaps to the shown value, or when the stage has different X and Y values,
    /// which this app shows read-only.
    public func commitStage(index: Int) {
        guard let stageDraft,
              stageDraft.index == index else {
            return
        }

        discardStageEdit()

        guard let snapshot,
              snapshot.stages.indices.contains(index) else {
            return
        }

        let stage = snapshot.stages[index]

        guard stage.x == stage.y,
              stageDraft.text != String(stage.x),
              let typed = UInt32(stageDraft.text) else {
            return
        }

        let dpi = nearestValidDpi(snapshot: snapshot, value: typed)

        guard dpi != stage.x else {
            return
        }

        send(.setStageDpi(index: UInt8(index), x: dpi, y: dpi))
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

}

private extension Command {

    /// Whether the mouse answers this command with an event that ends the pending state.
    var isWrite: Bool {
        switch self {
        case .setPolling, .setCurrentStage, .setStageCount, .setStageDpi, .setButton, .setShortcut:
            return true
        case .setBackgroundPolling, .refresh:
            return false
        }
    }

}
