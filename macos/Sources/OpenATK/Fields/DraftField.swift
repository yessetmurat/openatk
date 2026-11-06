//
//  DraftField.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// A numeric field. The typed text lives in the model, which drops it on every snapshot, failure
/// and write, so this field holds no value of its own.
struct DraftField: View {

    let model: MouseModel
    let field: FieldDraft.Field
    let title: String
    var suffix: String?

    @FocusState private var isFocused: Bool

    var body: some View {
        HStack(spacing: 1) {
            input
            if let suffix {
                Text(suffix)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 6)
        .padding(.vertical, 4)
        .frame(width: 80)
        .background(.quinary, in: RoundedRectangle(cornerRadius: 7))
        .overlay {
            RoundedRectangle(cornerRadius: 7)
                .strokeBorder(isFocused ? Color.accentColor : .clear, lineWidth: 2)
        }
    }

    // Plain, so a suffix can sit inside the box: a rounded border field has no room for one.
    private var input: some View {
        TextField(title, text: text)
            .textFieldStyle(.plain)
            .labelsHidden()
            .multilineTextAlignment(.trailing)
            .focused($isFocused)
            // Workaround, agreed with the owner: the model ignores edits during a write, but the
            // field would keep showing the typed text, because an ignored edit changes no value
            // and SwiftUI has nothing to push back. Swallowing key presses keeps the text from
            // changing at all. It does not cover paste or drag-and-drop of text, which can still
            // change the field during a write.
            .onKeyPress { press in
                model.isPending && press.key != .tab ? .handled : .ignored
            }
            .onSubmit {
                model.commitField(field)
            }
            .onExitCommand {
                model.discardFieldEdit()
            }
            .onChange(of: isFocused) {
                if !isFocused {
                    model.commitField(field)
                }
            }
    }

    private var text: Binding<String> {
        Binding(
            get: { model.fieldText(field) },
            set: { model.editField(field, text: $0) }
        )
    }

}
