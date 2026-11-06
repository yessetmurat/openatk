//
//  ButtonsView.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

/// One row per button. Every control shows the snapshot and only asks the model to write, so a
/// pick is not shown before the mouse confirms it. A change that would leave no left-click
/// button asks first.
struct ButtonsView: View {

    let model: MouseModel

    @State private var monitor = KeyCaptureMonitor()

    var body: some View {
        Group {
            if let snapshot = model.snapshot {
                form(snapshot)
            } else {
                Color.clear
            }
        }
        .onChange(of: model.captureButton) {
            if model.captureButton == nil {
                monitor.stop()
            } else {
                monitor.start(model: model) { button, keys in
                    model.request(button: button, change: .shortcut(keys))
                }
            }
        }
        .onDisappear {
            monitor.stop()
            model.cancelCapture()
        }
    }

    private func form(_ snapshot: SnapshotValue) -> some View {
        Form {
            Section {
                ForEach(snapshot.buttons, id: \.button) { value in
                    row(snapshot, value: value)
                }
            }
        }
        .formStyle(.grouped)
        .confirmationDialog(
            Strings.losesPrimaryClickTitle,
            isPresented: Binding(
                get: { model.confirmation != nil },
                set: { if !$0 { model.cancelConfirmation() } }
            ),
            titleVisibility: .visible,
            presenting: model.confirmation
        ) { request in
            Button(Strings.changeAnyway, role: .destructive) {
                model.confirm(request)
            }
            Button(Strings.cancel, role: .cancel) {}
        } message: { _ in
            Text(Strings.losesPrimaryClickMessage)
        }
    }

    private func row(_ snapshot: SnapshotValue, value: ButtonValue) -> some View {
        LabeledContent(Strings.button(value.button)) {
            if model.isEditable(button: value.button) {
                HStack(spacing: 8) {
                    detail(value)
                    picker(snapshot, value: value)
                }
            } else {
                Text(Strings.action(value.action))
                    .foregroundStyle(.secondary)
            }
        }
    }

    @ViewBuilder private func detail(_ value: ButtonValue) -> some View {
        if model.captureButton == value.button {
            VStack(alignment: .trailing) {
                Text(Strings.pressKey)
                if let message = model.captureMessage {
                    Text(message)
                        .foregroundStyle(.red)
                }
            }
        } else if value.action == .shortcut {
            Text(model.shortcutText(button: value.button))
                .foregroundStyle(.secondary)
            Button(Strings.changeKey) {
                model.beginCapture(button: value.button)
            }
        }
    }

    private func picker(_ snapshot: SnapshotValue, value: ButtonValue) -> some View {
        let selection = Binding(
            get: { value.action },
            set: { newAction in
                if newAction == .shortcut {
                    model.beginCapture(button: value.button)
                } else {
                    model.request(button: value.button, change: .action(newAction))
                }
            }
        )

        return Picker(Strings.button(value.button), selection: selection) {
            ForEach(model.actionChoices, id: \.self) { action in
                Text(Strings.action(action))
                    .tag(action)
            }
        }
        .labelsHidden()
        .pickerStyle(.menu)
        .fixedSize()
        .id(model.refreshCount)
    }

}
