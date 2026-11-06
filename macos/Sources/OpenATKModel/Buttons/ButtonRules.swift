//
//  ButtonRules.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// Rules about which button changes need a warning.
public enum ButtonRules {

    /// Whether giving `button` the new `action` leaves no button that clicks as the left button,
    /// when one did before. Only an action of `mouseLeft` counts as a left click.
    public static func losesPrimaryClick(
        buttons: [ButtonValue],
        button: MouseButton,
        action: Action
    ) -> Bool {
        guard let current = buttons.first(where: { $0.button == button }),
              current.action == .mouseLeft,
              action != .mouseLeft else {
            return false
        }

        return !buttons.contains { $0.button != button && $0.action == .mouseLeft }
    }

}
