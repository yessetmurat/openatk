//
//  Strings.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

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

    /// A battery charge.
    public static func batteryPercent(_ percent: UInt8) -> String {
        "\(percent)%"
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

    /// A stage whose axes differ.
    public static func splitDpi(x: UInt32, y: UInt32) -> String {
        "X \(x), Y \(y)"
    }

}
