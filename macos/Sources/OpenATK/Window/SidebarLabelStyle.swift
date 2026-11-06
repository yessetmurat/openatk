//
//  SidebarLabelStyle.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import SwiftUI

/// A sidebar row: a fixed icon column, then the title. It replaces the sidebar's automatic label
/// style, which sets the title of the selected row bold from inside the label.
struct SidebarLabelStyle: LabelStyle {

    private static let iconSize: CGFloat = 16
    private static let iconColumnWidth: CGFloat = 22
    private static let spacing: CGFloat = 6

    func makeBody(configuration: Configuration) -> some View {
        HStack(spacing: Self.spacing) {
            configuration.icon
                .font(.system(size: Self.iconSize))
                .frame(width: Self.iconColumnWidth)

            configuration.title
        }
    }

}
