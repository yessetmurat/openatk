//
//  PerformanceView.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

/// Polling rate and DPI stages. Every control shows the snapshot and only asks the model to
/// write, so a pick is not shown before the mouse confirms it. Nothing is disabled during a
/// write, so the window does not flicker: the model refuses sends while one is in flight, and
/// the getters read the snapshot, so a click then changes nothing on screen.
struct PerformanceView: View {

    let model: MouseModel

    var body: some View {
        Group {
            if let snapshot = model.snapshot {
                form(snapshot)
            } else {
                Color.clear
            }
        }
        .onDisappear {
            model.discardFieldEdit()
        }
    }

    private func form(_ snapshot: SnapshotValue) -> some View {
        Form {
            Section {
                pollingPicker(snapshot)
            }

            Section(Strings.dpiStages) {
                stageCountStepper(snapshot)

                ForEach(snapshot.stages.indices, id: \.self) { index in
                    stageRow(snapshot, index: index)
                }
            }
        }
        .formStyle(.grouped)
    }

    private func pollingPicker(_ snapshot: SnapshotValue) -> some View {
        let selection = Binding(
            get: { snapshot.pollingHz },
            set: { model.selectPolling(hz: $0) }
        )

        return Picker(Strings.pollingRate, selection: selection) {
            ForEach(snapshot.pollingOptionsHz, id: \.self) { hertz in
                Text(Strings.hertz(hertz))
                    .tag(hertz)
            }

            if !snapshot.pollingOptionsHz.contains(snapshot.pollingHz) {
                Text(Strings.hertz(snapshot.pollingHz))
                    .selectionDisabled()
                    .tag(snapshot.pollingHz)
            }
        }
        .pickerStyle(.menu)
        // Safe only while no other row of this section has an `.id`: see `SensorView.rowID`.
        .id(model.refreshCount)
    }

    private func stageCountStepper(_ snapshot: SnapshotValue) -> some View {
        let count = snapshot.stages.count
        let range = MouseModel.stageCountRange

        return LabeledContent(Strings.stageCount) {
            HStack(spacing: 8) {
                Text(String(count))
                Stepper(
                    Strings.stageCount,
                    onIncrement: count < range.upperBound ? { model.changeStageCount(by: 1) } : nil,
                    onDecrement: count > range.lowerBound ? { model.changeStageCount(by: -1) } : nil
                )
                .labelsHidden()
                .id(model.refreshCount)
            }
        }
    }

    private func stageRow(_ snapshot: SnapshotValue, index: Int) -> some View {
        let stage = snapshot.stages[index]
        let selection = Binding(
            get: { Int(snapshot.currentStage) },
            set: { model.selectStage(index: $0) }
        )

        return LabeledContent {
            if stage.x == stage.y {
                DraftField(model: model, field: .stage(index), title: Strings.stage(index: index))
            } else {
                Text(Strings.splitDpi(x: stage.x, y: stage.y))
            }
        } label: {
            // A one-item radio group: the native radio button, one per row.
            Picker(Strings.stage(index: index), selection: selection) {
                Text(Strings.stage(index: index))
                    .tag(index)
            }
            .pickerStyle(.radioGroup)
            .labelsHidden()
            .id(model.refreshCount)
        }
    }

}
