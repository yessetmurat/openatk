//
//  Header.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

/// The mouse's name, profile picker and battery, or the not-connected text. The picker shows the
/// snapshot and only asks the model to write, so a pick is not shown before the mouse confirms it.
struct Header: View {

    let model: MouseModel

    var body: some View {
        HStack(spacing: 16) {
            if let snapshot = model.snapshot {
                Text(snapshot.modelName)
                    .fontWeight(.semibold)
                Spacer()
                profilePicker(snapshot)
                battery(snapshot)
            } else {
                Text(Strings.notConnected)
                    .foregroundStyle(.secondary)
                Spacer()
            }
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
    }

    private func profilePicker(_ snapshot: SnapshotValue) -> some View {
        let selection = Binding(
            get: { snapshot.profile },
            set: { model.selectProfile(index: $0) }
        )

        return Picker(Strings.profileLabel, selection: selection) {
            ForEach(snapshot.profileChoices, id: \.value) { choice in
                Text(Strings.profile(index: choice.value))
                    .selectionDisabled(!choice.isSelectable)
                    .tag(choice.value)
            }
        }
        .labelsHidden()
        .pickerStyle(.menu)
        // Borderless keeps the row at the height of its text; a bordered menu adds 7 points.
        .buttonStyle(.borderless)
        .fixedSize()
        .id(model.refreshCount)
    }

    private func battery(_ snapshot: SnapshotValue) -> some View {
        HStack(spacing: 4) {
            switch snapshot.batteryDisplay {
            case .charging:
                Text(Strings.charging)
                Image(systemName: "bolt.fill")
            case .percent(let percent):
                Text(Strings.batteryPercent(percent))
            }
        }
        .foregroundStyle(.secondary)
    }

}
