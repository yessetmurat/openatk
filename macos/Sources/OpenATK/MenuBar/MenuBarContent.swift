//
//  MenuBarContent.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import AppKit
import OpenATKModel
import SwiftUI

/// The native menu: maps the entries of `MenuContent` to items. A checkable item shows the
/// snapshot and only asks the model to write, so a pick is not shown before the mouse confirms it.
struct MenuBarContent: View {

    let delegate: AppDelegate

    var body: some View {
        let entries = MenuContent.entries(snapshot: delegate.model.snapshot, strings: Strings.self)

        ForEach(entries.indices, id: \.self) { index in
            item(entries[index])
        }
    }

    @ViewBuilder private func item(_ entry: MenuEntry) -> some View {
        switch entry {
        case .info(let title):
            Text(title)
        case .submenu(let title, let items):
            Menu(title) {
                ForEach(items.indices, id: \.self) { index in
                    toggle(items[index])
                }
            }
        case .separator:
            Divider()
        case .openWindow(let title):
            Button(title) {
                delegate.showMainWindow()
            }
        case .quit(let title):
            Button(title) {
                NSApp.terminate(nil)
            }
        }
    }

    private func toggle(_ choice: MenuChoice) -> some View {
        let isOn = Binding(
            get: { choice.isChecked },
            set: { _ in delegate.model.pick(choice) }
        )

        return Toggle(choice.title, isOn: isOn)
            .disabled(!choice.isEnabled)
    }

}
