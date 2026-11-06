//
//  MenuBarLabel.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The mouse symbol beside the battery. It lives as long as the menu bar item, also with the
/// window closed, so it also hands the delegate the action that opens the window.
struct MenuBarLabel: View {

    let delegate: AppDelegate

    @Environment(\.openWindow) private var openWindow

    var body: some View {
        HStack {
            Image(systemName: "computermouse")
            Text(MenuContent.title(snapshot: delegate.model.snapshot))
        }
        .onAppear {
            delegate.openMainWindow = {
                openWindow(id: MainWindow.sceneID)
            }
        }
    }

}
