//
//  MouseModel+Menu.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

extension MouseModel {

    /// Asks for what a menu item selects. The menu keeps showing the snapshot until the mouse
    /// answers.
    public func pick(_ choice: MenuChoice) {
        switch choice.target {
        case .stage(let index):
            selectStage(index: index)
        case .rate(let hertz):
            selectPolling(hz: hertz)
        }
    }

}
