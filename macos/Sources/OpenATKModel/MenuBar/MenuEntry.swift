//
//  MenuEntry.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// One line of the menu bar menu.
public enum MenuEntry: Equatable {

    /// Text that cannot be picked.
    case info(String)

    /// A native submenu of checkable values, titled with the current one.
    case submenu(title: String, items: [MenuChoice])

    /// A line between groups.
    case separator

    /// Opens the window.
    case openWindow(String)

    /// Quits the app.
    case quit(String)

}
