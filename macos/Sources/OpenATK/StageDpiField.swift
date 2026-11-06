//
//  StageDpiField.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The DPI of one stage. The typed text lives in the model, which drops it on every snapshot,
/// failure and write, so this field holds no value of its own.
struct StageDpiField: View {

    let model: MouseModel
    let index: Int

    @FocusState private var isFocused: Bool

    var body: some View {
        TextField(Strings.stage(index: index), text: text)
            .textFieldStyle(.roundedBorder)
            .labelsHidden()
            .multilineTextAlignment(.trailing)
            .frame(width: 80)
            .focused($isFocused)
            // The model ignores edits during a write, but the field would keep showing the typed
            // text: an ignored edit changes no value, so SwiftUI has nothing to push back.
            .onKeyPress { press in
                model.isPending && press.key != .tab ? .handled : .ignored
            }
            .onSubmit {
                model.commitStage(index: index)
            }
            .onExitCommand {
                model.discardStageEdit()
            }
            .onChange(of: isFocused) {
                if !isFocused {
                    model.commitStage(index: index)
                }
            }
    }

    private var text: Binding<String> {
        Binding(
            get: { model.stageText(index: index) },
            set: { model.editStage(index: index, text: $0) }
        )
    }

}
