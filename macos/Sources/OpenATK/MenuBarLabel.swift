//
//  MenuBarLabel.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The mouse symbol beside the battery. It lives as long as the app, also with the window closed
/// and with the icon turned off, so it also hands the delegate the action that opens the window.
struct MenuBarLabel: View {

    let delegate: AppDelegate

    @Environment(\.openWindow) private var openWindow

    var body: some View {
        HStack {
            Image(systemName: "computermouse")
            Text(MenuContent.title(snapshot: delegate.model.snapshot))
        }
        .onAppear {
            delegate.windowOpenerReady {
                openWindow(id: MainWindow.sceneID)
            }
        }
    }

}
