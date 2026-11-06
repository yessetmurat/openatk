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

    /// The Sensor section.
    public static let sensor = "Sensor"

    /// The Settings section.
    public static let settings = "Settings"

    /// Shown in place of the mouse's name when none can be reached.
    public static let notConnected = "Mouse not connected"

    /// Shown, and spoken for the charging mark, while the mouse charges.
    public static let charging = "Charging"

    /// Names the profile picker, whose label is hidden, for VoiceOver.
    public static let profileLabel = "Profile"

    /// The action that switches to the next profile.
    public static let cycleProfile = "Cycle profile"

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

    /// Label of the mouse's firmware version.
    public static let firmware = "Firmware"

    /// Label of the dongle's firmware version.
    public static let dongleFirmware = "Dongle firmware"

    /// Label of the launch at login toggle.
    public static let launchAtLogin = "Launch at login"

    /// Shown while the login item waits for the user's approval.
    public static let launchAtLoginNeedsApproval =
        "Allow openatk in System Settings, under General, Login Items."

    /// Shown in place of a setting the mouse did not report.
    public static let unknown = "Unknown"

    /// Label of the lift-off picker.
    public static let liftOff = "Lift-off distance"

    /// Label of the debounce picker.
    public static let debounce = "Debounce time"

    /// Warns about a low debounce time.
    public static let debounceWarning = "Below 4 ms, a single click can register as two."

    /// Label of the motion sync toggle.
    public static let motionSync = "Motion sync"

    /// Label of the linear correction toggle.
    public static let linearCorrection = "Linear correction"

    /// Label of the ripple control toggle.
    public static let rippleControl = "Ripple control"

    /// Label of the sensor angle row.
    public static let angle = "Sensor angle"

    /// The unit beside the angle field.
    public static let degreeSign = "°"

    /// Label of the sleep picker.
    public static let sleepTimer = "Sleep timer"

    /// Label of the far distance toggle.
    public static let farDistance = "Far distance mode"

    /// Why launch at login could not be changed.
    public static func launchAtLoginFailed(reason: String) -> String {
        "Could not change launch at login: \(reason)"
    }

    /// A lift-off height.
    public static func liftOffHeight(tenths: UInt8) -> String {
        "\(tenths / 10).\(tenths % 10) mm"
    }

    /// A debounce time.
    public static func milliseconds(_ milliseconds: UInt8) -> String {
        "\(milliseconds) ms"
    }

    /// A sleep time: whole minutes as minutes, anything else as seconds.
    public static func sleepTime(seconds: UInt16) -> String {
        if seconds > 0,
           seconds.isMultiple(of: 60) {
            return "\(seconds / 60) min"
        }

        return "\(seconds) s"
    }

    /// A battery charge.
    public static func batteryPercent(_ percent: UInt8) -> String {
        "\(percent)%"
    }

    /// The battery line of the menu.
    public static func battery(_ display: BatteryDisplay) -> String {
        switch display {
        case .charging:
            "Battery: \(charging)"
        case .percent(let percent):
            "Battery: \(batteryPercent(percent))"
        }
    }

    /// The active profile at `index`, counted from 0 and shown from 1. The byte comes from the
    /// mouse unchecked, so it is widened before adding.
    public static func profile(index: UInt8) -> String {
        "Profile \(Int(index) + 1)"
    }

    /// The title of the profile submenu, with the active profile shown from 1.
    public static func profileTitle(index: UInt8) -> String {
        "Profile: \(Int(index) + 1)"
    }

    /// The title of the polling submenu.
    public static func pollingTitle(hertz: UInt32) -> String {
        "Polling: \(self.hertz(hertz))"
    }

    /// The title of the DPI submenu, or a dash when the current stage is not among the stages.
    public static func dpiTitle(stage: StageValue?) -> String {
        "DPI: \(stage.map(stageValue) ?? noValue)"
    }

    /// A polling rate.
    public static func hertz(_ hertz: UInt32) -> String {
        "\(hertz) Hz"
    }

    /// The stage at `index`, counted from 0 and shown from 1.
    public static func stage(index: Int) -> String {
        "Stage \(index + 1)"
    }

    /// A stage as a menu item.
    public static func dpi(_ stage: StageValue) -> String {
        "\(stageValue(stage)) DPI"
    }

    /// A stage's value: one number, or X / Y when the axes differ.
    private static func stageValue(_ stage: StageValue) -> String {
        stage.x == stage.y ? "\(stage.x)" : "\(stage.x) / \(stage.y)"
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
        case .profile(let index):
            profile(index: index)
        case .profileCycle:
            cycleProfile
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
