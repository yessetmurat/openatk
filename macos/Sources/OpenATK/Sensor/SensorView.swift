//
//  SensorView.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

/// The sensor settings. Every control shows the snapshot and only asks the model to write, so a
/// pick is not shown before the mouse confirms it. A setting the mouse did not report shows as
/// unknown, with no control.
struct SensorView: View {

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
                picker(
                    Strings.liftOff,
                    value: snapshot.liftOffTenths,
                    choices: model.liftOffChoices,
                    text: Strings.liftOffHeight(tenths:),
                    select: model.selectLiftOff(tenths:)
                )
                picker(
                    Strings.debounce,
                    value: snapshot.debounceMs,
                    choices: model.debounceChoices,
                    text: Strings.milliseconds,
                    select: model.selectDebounce(ms:)
                )
            } footer: {
                if model.showsDebounceWarning {
                    Text(Strings.debounceWarning)
                        .foregroundStyle(.secondary)
                }
            }

            Section {
                toggle(
                    Strings.motionSync,
                    value: snapshot.motionSync,
                    select: model.selectMotionSync(enabled:)
                )
                toggle(
                    Strings.linearCorrection,
                    value: snapshot.linearCorrection,
                    select: model.selectLinearCorrection(enabled:)
                )
                toggle(
                    Strings.rippleControl,
                    value: snapshot.rippleControl,
                    select: model.selectRippleControl(enabled:)
                )

                if model.showsAngle,
                   let angle = snapshot.angle {
                    angleRow(angle)
                }

                picker(
                    Strings.sleepTimer,
                    value: snapshot.sleepSeconds,
                    choices: model.sleepChoices,
                    text: Strings.sleepTime(seconds:),
                    select: model.selectSleep(seconds:)
                )
                toggle(
                    Strings.farDistance,
                    value: snapshot.farDistance,
                    select: model.selectFarDistance(enabled:)
                )
            }
        }
        .formStyle(.grouped)
    }

    /// The value and its switch side by side: an angle has no effect while the switch is off.
    private func angleRow(_ angle: AngleValue) -> some View {
        LabeledContent(Strings.angle) {
            HStack(spacing: 8) {
                if angle.degrees != nil {
                    DraftField(model: model, field: .angle, title: Strings.angle)
                    Text(Strings.degreeSign)
                }

                if let enabled = angle.enabled {
                    Toggle(Strings.angle, isOn: binding(enabled, select: model.selectAngleEnabled))
                        .labelsHidden()
                        .toggleStyle(.switch)
                        .id(model.refreshCount)
                }

                if angle.degrees == nil || angle.enabled == nil {
                    Text(Strings.unknown)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    @ViewBuilder private func picker<Value: Hashable & Sendable>(
        _ title: String,
        value: Value?,
        choices: [PickerChoice<Value>],
        text: @escaping (Value) -> String,
        select: @escaping @MainActor (Value) -> Void
    ) -> some View {
        if let value {
            Picker(title, selection: binding(value, select: select)) {
                ForEach(choices, id: \.value) { choice in
                    Text(text(choice.value))
                        .selectionDisabled(!choice.isSelectable)
                        .tag(choice.value)
                }
            }
            .pickerStyle(.menu)
            .id(rowID(title))
        } else {
            LabeledContent(title, value: Strings.unknown)
        }
    }

    @ViewBuilder private func toggle(
        _ title: String,
        value: Bool?,
        select: @escaping @MainActor (Bool) -> Void
    ) -> some View {
        if let value {
            Toggle(title, isOn: binding(value, select: select))
                .id(rowID(title))
        } else {
            LabeledContent(title, value: Strings.unknown)
        }
    }

    /// The identity of a row: new on each refresh, so a control changed without effect is rebuilt,
    /// and unlike its neighbours', since a `Form` draws rows of equal identity as the first one.
    private func rowID(_ title: String) -> String {
        "\(title) \(model.refreshCount)"
    }

    /// A binding that always reads `value`, the snapshot's, and only asks the model to change it.
    private func binding<Value: Sendable>(
        _ value: Value,
        select: @escaping @MainActor (Value) -> Void
    ) -> Binding<Value> {
        Binding(
            get: { value },
            set: { select($0) }
        )
    }

}
