//
//  Strings.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// All user-facing text, in one place so another language can be added.
public enum Strings {

    /// The application name.
    public static let appName = "openatk"

    /// The Performance section.
    public static let performance = "Performance"

    /// The Buttons section.
    public static let buttons = "Buttons"

    /// The Settings section.
    public static let settings = "Settings"

    /// Shown in place of the mouse's name when none can be reached.
    public static let notConnected = "Mouse not connected"

    /// Spoken for the charging mark.
    public static let charging = "Charging"

    /// Label of the polling rate menu.
    public static let pollingRate = "Polling rate"

    /// Title of the DPI stages group.
    public static let dpiStages = "DPI stages"

    /// Label of the stage count stepper.
    public static let stageCount = "Number of stages"

    /// Stands in for a value that cannot be read.
    public static let noValue = "--"

    /// Menu item that brings up the window.
    public static let openFullWindow = "Open full window"

    /// Menu item that quits the app.
    public static let quit = "Quit openatk"

    /// Label of the menu bar icon toggle.
    public static let showMenuBarIcon = "Show menu bar icon"

    /// Label of the app version.
    public static let version = "Version"

    /// A battery charge.
    public static func batteryPercent(_ percent: UInt8) -> String {
        "\(percent)%"
    }

    /// The battery line of the menu.
    public static func battery(percent: UInt8) -> String {
        "Battery: \(batteryPercent(percent))"
    }

    /// The active profile at `index`, counted from 0 and shown from 1. The byte comes from the
    /// mouse unchecked, so it is widened before adding.
    public static func profile(index: UInt8) -> String {
        "Profile \(Int(index) + 1)"
    }

    /// A polling rate.
    public static func hertz(_ hertz: UInt32) -> String {
        "\(hertz) Hz"
    }

    /// The stage at `index`, counted from 0 and shown from 1.
    public static func stage(index: Int) -> String {
        "Stage \(index + 1)"
    }

    /// A stage whose axes are equal.
    public static func dpi(_ dpi: UInt32) -> String {
        "\(dpi) DPI"
    }

    /// A stage whose axes differ.
    public static func splitDpi(x: UInt32, y: UInt32) -> String {
        "X \(x), Y \(y)"
    }

    /// Shown for a shortcut whose keys the mouse did not report in an editable shape.
    public static let unknownShortcut = "Unknown shortcut"

    /// The picker entry for a keyboard shortcut.
    public static let keyboardKey = "Keyboard key"

    /// Prompt while a key is being captured.
    public static let pressKey = "Press a key, Esc to cancel"

    /// Button that re-opens capture.
    public static let changeKey = "Change key"

    /// Shown when the pressed key has no mouse usage.
    public static let unmappedKey = "That key cannot be set on the mouse."

    /// Shown when more than two modifiers are held.
    public static let tooManyModifiers = "Use at most two modifier keys."

    /// Title of the left-click warning.
    public static let losesPrimaryClickTitle = "No button will left-click"

    /// Body of the left-click warning.
    public static let losesPrimaryClickMessage =
        "This was the only left-click button. Without one, you may not be able to click."

    /// Confirms the warned change.
    public static let changeAnyway = "Change anyway"

    /// Cancels a warned change.
    public static let cancel = "Cancel"

    /// A button of the mouse.
    public static func button(_ button: MouseButton) -> String {
        switch button {
        case .left:
            "Left button"
        case .right:
            "Right button"
        case .middle:
            "Wheel button"
        case .side1:
            "Side button 1"
        case .side2:
            "Side button 2"
        }
    }

    /// An action of a button.
    public static func action(_ action: Action) -> String {
        switch action {
        case .disabled:
            "Disabled"
        case .mouseLeft:
            "Left click"
        case .mouseRight:
            "Right click"
        case .mouseMiddle:
            "Middle click"
        case .mouseSide1:
            "Back"
        case .mouseSide2:
            "Forward"
        case .dpiCycle:
            "DPI cycle"
        case .dpiPlus:
            "DPI up"
        case .dpiMinus:
            "DPI down"
        case .wheelUp:
            "Scroll up"
        case .wheelDown:
            "Scroll down"
        case .shortcut:
            keyboardKey
        case .other(let actionClass, _, _):
            "Other (class \(actionClass))"
        }
    }

    /// The name of a key that is a word, by HID usage, or `nil` for any other key.
    static func keyWord(usage: UInt8) -> String? {
        switch usage {
        case 0x28:
            "Return"
        case 0x29:
            "Esc"
        case 0x2a:
            "Delete"
        case 0x2b:
            "Tab"
        case 0x2c:
            "Space"
        case 0x4a:
            "Home"
        case 0x4b:
            "Page Up"
        case 0x4c:
            "Forward Delete"
        case 0x4d:
            "End"
        case 0x4e:
            "Page Down"
        case 0x4f:
            "Right"
        case 0x50:
            "Left"
        case 0x51:
            "Down"
        case 0x52:
            "Up"
        default:
            nil
        }
    }

    /// A modifier key, by its HID mask.
    static func modifier(mask: UInt8) -> String {
        switch mask {
        case 1:
            "Ctrl"
        case 2:
            "Shift"
        case 4:
            "Alt"
        case 8:
            "Cmd"
        default:
            "Modifier 0x" + String(mask, radix: 16)
        }
    }

    /// A key with no name in the table.
    static func unnamedKey(usage: UInt8) -> String {
        "Key 0x" + String(usage, radix: 16)
    }

}
