//
//  MenuBarLabel.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The mouse symbol beside the battery.
struct MenuBarLabel: View {

    let delegate: AppDelegate

    var body: some View {
        HStack {
            Image(systemName: "computermouse")

            if MenuContent.isCharging(snapshot: delegate.model.snapshot) {
                Image(systemName: "bolt.fill")
                    .accessibilityLabel(Strings.charging)
            } else {
                Text(MenuContent.title(snapshot: delegate.model.snapshot))
            }
        }
    }

}
