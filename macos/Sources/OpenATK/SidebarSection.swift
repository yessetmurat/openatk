//
//  SidebarSection.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel

/// The sections of the window, in sidebar order.
enum SidebarSection: CaseIterable {

    case performance
    case buttons
    case settings

    var title: String {
        switch self {
        case .performance:
            Strings.performance
        case .buttons:
            Strings.buttons
        case .settings:
            Strings.settings
        }
    }

    var symbol: String {
        switch self {
        case .performance:
            "gauge.with.dots.needle.67percent"
        case .buttons:
            "computermouse"
        case .settings:
            "gearshape"
        }
    }

}
