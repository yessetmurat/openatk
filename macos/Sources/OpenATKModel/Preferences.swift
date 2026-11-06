//
//  Preferences.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import Foundation
import Observation

/// The app's own settings, kept in `UserDefaults`. Nothing here is stored on the mouse.
@MainActor
@Observable
public final class Preferences {

    private static let menuBarIconKey = "menuBarIcon"

    /// Whether the menu bar icon is shown, which also keeps the app alive without its window.
    ///
    /// `MenuBarExtra` writes its binding back on every update of its label, with the same value.
    public var menuBarIcon: Bool {
        didSet {
            guard menuBarIcon != oldValue else {
                return
            }

            defaults.set(menuBarIcon, forKey: Self.menuBarIconKey)
            pollingModel?.setBackgroundPolling(menuBarIcon)
        }
    }

    @ObservationIgnored private let defaults: UserDefaults
    @ObservationIgnored private var pollingModel: MouseModel?

    /// Reads the settings from `defaults`. The icon is on until the user turns it off.
    public init(defaults: UserDefaults) {
        self.defaults = defaults
        menuBarIcon = defaults.object(forKey: Self.menuBarIconKey) as? Bool ?? true
    }

    /// Whether the app shows in the Dock. With the window closed it lives in the menu bar alone;
    /// without the menu bar icon, closing the window quits, so the Dock icon stays until then.
    public func showsDockIcon(windowOpen: Bool) -> Bool {
        windowOpen || !menuBarIcon
    }

    /// Sets `model`'s background polling to ``menuBarIcon``, now and on every later change.
    public func applyBackgroundPolling(to model: MouseModel) {
        pollingModel = model
        model.setBackgroundPolling(menuBarIcon)
    }

}
