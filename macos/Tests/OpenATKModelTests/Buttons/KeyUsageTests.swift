//
//  KeyUsageTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

struct KeyUsageTests {

    @Test func lettersMapToUsages() {
        #expect(KeyUsage.fromKeyCode(0) == 0x04)
        #expect(KeyUsage.fromKeyCode(6) == 0x1d)
    }

    @Test func digitsMap() {
        #expect(KeyUsage.fromKeyCode(18) == 0x1e)
        #expect(KeyUsage.fromKeyCode(29) == 0x27)
    }

    @Test func functionKeysMap() {
        #expect(KeyUsage.fromKeyCode(122) == 0x3a)
    }

    @Test func homeAndEndMap() {
        #expect(KeyUsage.fromKeyCode(115) == 0x4a)
        #expect(KeyUsage.fromKeyCode(119) == 0x4d)
    }

    @Test func enterEscapeSpaceMap() {
        #expect(KeyUsage.fromKeyCode(36) == 0x28)
        #expect(KeyUsage.fromKeyCode(53) == 0x29)
        #expect(KeyUsage.fromKeyCode(49) == 0x2c)
    }

    @Test func unmappedKeyIsNil() {
        #expect(KeyUsage.fromKeyCode(0xFFFF) == nil)
    }

    @Test func commandIsMask8() {
        #expect(KeyUsage.modifiers([.command]) == [.modifier(mask: 8)])
    }

    @Test func modifiersComeInMaskOrder() {
        #expect(KeyUsage.modifiers([.command, .control]) == [.modifier(mask: 1), .modifier(mask: 8)])
    }

    @Test func namesComeFromTheTable() {
        #expect(KeyUsage.name(usage: 0x06) == "C")
        #expect(KeyUsage.name(usage: 0x4d) == "End")
        #expect(KeyUsage.name(usage: 0xFE) == nil)
    }

    @Test(arguments: [
        (0, 0x04), (11, 0x05), (8, 0x06), (2, 0x07), (14, 0x08), (3, 0x09), (5, 0x0a), (4, 0x0b),
        (34, 0x0c), (38, 0x0d), (40, 0x0e), (37, 0x0f), (46, 0x10), (45, 0x11), (31, 0x12),
        (35, 0x13), (12, 0x14), (15, 0x15), (1, 0x16), (17, 0x17), (32, 0x18), (9, 0x19),
        (13, 0x1a), (7, 0x1b), (16, 0x1c), (6, 0x1d),
        (18, 0x1e), (19, 0x1f), (20, 0x20), (21, 0x21), (23, 0x22), (22, 0x23), (26, 0x24),
        (28, 0x25), (25, 0x26), (29, 0x27),
        (36, 0x28), (53, 0x29), (51, 0x2a), (48, 0x2b), (49, 0x2c), (27, 0x2d), (24, 0x2e),
        (33, 0x2f), (30, 0x30), (42, 0x31), (41, 0x33), (39, 0x34), (50, 0x35), (43, 0x36),
        (47, 0x37), (44, 0x38),
        (122, 0x3a), (120, 0x3b), (99, 0x3c), (118, 0x3d), (96, 0x3e), (97, 0x3f), (98, 0x40),
        (100, 0x41), (101, 0x42), (109, 0x43), (103, 0x44), (111, 0x45),
        (115, 0x4a), (116, 0x4b), (117, 0x4c), (119, 0x4d), (121, 0x4e), (124, 0x4f), (123, 0x50),
        (125, 0x51), (126, 0x52),
    ] as [(UInt16, UInt8)])
    func everyKeyCodeMapsAndHasAName(code: UInt16, usage: UInt8) {
        #expect(KeyUsage.fromKeyCode(code) == usage)
        #expect(KeyUsage.name(usage: usage) != nil)
    }

}
