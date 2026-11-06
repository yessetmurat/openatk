//
//  KeyUsage.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// Maps macOS virtual key codes to the HID usages the mouse stores, and back to names.
public enum KeyUsage {

    /// HID modifier masks, in the order they are listed: Ctrl 1, Shift 2, Alt 4, Cmd 8.
    static let modifierMasks: [(modifier: KeyModifiers, mask: UInt8)] = [
        (.control, 1), (.shift, 2), (.option, 4), (.command, 8),
    ]

    private static let usageByKeyCode: [UInt16: UInt8] = {
        var table: [UInt16: UInt8] = [:]

        // Letters A to Z, by their ANSI key codes.
        let letters: [UInt16] = [
            0, 11, 8, 2, 14, 3, 5, 4, 34, 38, 40, 37, 46,
            45, 31, 35, 12, 15, 1, 17, 32, 9, 13, 7, 16, 6,
        ]
        for (offset, code) in letters.enumerated() {
            table[code] = 0x04 + UInt8(offset)
        }

        // Digits 1 to 9, then 0.
        let digits: [UInt16] = [18, 19, 20, 21, 23, 22, 26, 28, 25, 29]
        for (offset, code) in digits.enumerated() {
            table[code] = 0x1e + UInt8(offset)
        }

        // F1 to F12.
        let functionKeys: [UInt16] = [122, 120, 99, 118, 96, 97, 98, 100, 101, 109, 103, 111]
        for (offset, code) in functionKeys.enumerated() {
            table[code] = 0x3a + UInt8(offset)
        }

        let others: [UInt16: UInt8] = [
            36: 0x28, 53: 0x29, 51: 0x2a, 48: 0x2b, 49: 0x2c, 27: 0x2d, 24: 0x2e, 33: 0x2f,
            30: 0x30, 42: 0x31, 41: 0x33, 39: 0x34, 50: 0x35, 43: 0x36, 47: 0x37, 44: 0x38,
            115: 0x4a, 116: 0x4b, 117: 0x4c, 119: 0x4d, 121: 0x4e, 124: 0x4f, 123: 0x50,
            125: 0x51, 126: 0x52,
        ]
        table.merge(others) { current, _ in current }

        return table
    }()

    // Labels follow US key positions; the usage is the same on every layout.
    private static let nameByUsage: [UInt8: String] = {
        var table: [UInt8: String] = [:]

        for (offset, letter) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".enumerated() {
            table[0x04 + UInt8(offset)] = String(letter)
        }

        for (offset, digit) in "1234567890".enumerated() {
            table[0x1e + UInt8(offset)] = String(digit)
        }

        for number in 1...12 {
            table[0x39 + UInt8(number)] = "F\(number)"
        }

        let symbols: [UInt8: String] = [
            0x2d: "-", 0x2e: "=", 0x2f: "[", 0x30: "]", 0x31: "\\", 0x33: ";", 0x34: "'",
            0x35: "`", 0x36: ",", 0x37: ".", 0x38: "/",
        ]
        table.merge(symbols) { current, _ in current }

        return table
    }()

    /// The HID usage of a macOS virtual key code, or `nil` for a key the mouse cannot send.
    public static func fromKeyCode(_ code: UInt16) -> UInt8? {
        usageByKeyCode[code]
    }

    /// The held modifiers as HID modifier keys, in mask order.
    public static func modifiers(_ modifiers: KeyModifiers) -> [ShortcutKeyValue] {
        modifierMasks
            .filter { modifiers.contains($0.modifier) }
            .map { .modifier(mask: $0.mask) }
    }

    /// The name of a HID usage, or `nil` when it has none.
    public static func name(usage: UInt8) -> String? {
        nameByUsage[usage] ?? Strings.keyWord(usage: usage)
    }

}

extension KeyUsage {

    /// Whether `keys` is a shape the mouse accepts: one key, or one or two modifiers plus one key.
    static func isValidShortcut(_ keys: [ShortcutKeyValue]) -> Bool {
        var modifiers = 0
        var generals = 0

        for key in keys {
            switch key {
            case .modifier:
                modifiers += 1
            case .general:
                generals += 1
            }
        }

        return generals == 1 && modifiers <= 2
    }

    /// The text of one key: a modifier's name, or the usage's name.
    static func text(_ key: ShortcutKeyValue) -> String {
        switch key {
        case .modifier(let mask):
            return Strings.modifier(mask: mask)
        case .general(let usage):
            return name(usage: usage) ?? Strings.unnamedKey(usage: usage)
        }
    }

}
