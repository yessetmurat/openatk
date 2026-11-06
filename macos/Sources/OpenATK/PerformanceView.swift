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
/// write, so a pick is not shown before the mouse confirms it. A write disables the pickers and
/// the stepper but not the DPI fields: disabling a field drops its focus, which would end
/// Tab-through editing, and the model already ignores typing while a write is in flight.
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
            model.discardStageEdit()
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
        .disabled(model.isPending)
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
                .disabled(model.isPending)
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
                StageDpiField(model: model, index: index)
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
            .disabled(model.isPending)
        }
    }

}
