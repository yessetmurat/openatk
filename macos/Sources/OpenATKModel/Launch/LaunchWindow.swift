//
//  LaunchWindow.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

/// Whether the window opens at launch, and when. The scene never opens it by itself. The decision
/// comes from the app delegate and the way to open comes from the app's scene body, in either
/// order, so whichever arrives second opens the window.
public struct LaunchWindow {

    /// Whether this start shows the window. A login start stays in the menu bar, so without the
    /// icon the window opens: the app would have nothing on screen otherwise.
    public private(set) var wantsWindow = false

    private var hasClosure = false
    private var hasOpened = false

    /// Creates a value with no decision and no way to open yet.
    public init() {}

    /// Records how the app started. Returns whether to open the window now.
    public mutating func decide(isLoginStart: Bool, menuBarIcon: Bool) -> Bool {
        wantsWindow = !isLoginStart || !menuBarIcon

        return opensNow()
    }

    /// Records that the window can be opened. Returns whether to open it now.
    public mutating func closureReady() -> Bool {
        hasClosure = true

        return opensNow()
    }

    private mutating func opensNow() -> Bool {
        guard wantsWindow,
              hasClosure,
              !hasOpened else {
            return false
        }

        hasOpened = true

        return true
    }

}
