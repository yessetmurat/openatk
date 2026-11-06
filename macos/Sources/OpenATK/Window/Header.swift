//
//  Header.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

/// The mouse's name, profile and battery, or the not-connected text.
struct Header: View {

    let snapshot: SnapshotValue?

    var body: some View {
        HStack(spacing: 16) {
            if let snapshot {
                Text(snapshot.modelName)
                    .fontWeight(.semibold)
                Spacer()
                Text(Strings.profile(index: snapshot.profile))
                    .foregroundStyle(.secondary)
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

    private func battery(_ snapshot: SnapshotValue) -> some View {
        HStack(spacing: 4) {
            Text(Strings.batteryPercent(snapshot.percent))

            if snapshot.charging {
                Image(systemName: "bolt.fill")
                    .accessibilityLabel(Strings.charging)
            }
        }
        .foregroundStyle(.secondary)
    }

}
