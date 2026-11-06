//
//  ButtonRulesTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

struct ButtonRulesTests {

    private static func buttons(left: Action, side1: Action) -> [ButtonValue] {
        [
            ButtonValue(button: .left, action: left, keys: nil),
            ButtonValue(button: .right, action: .mouseRight, keys: nil),
            ButtonValue(button: .side1, action: side1, keys: nil),
        ]
    }

    @Test func removingLastLeftClickIsFlagged() {
        let buttons = Self.buttons(left: .mouseLeft, side1: .dpiCycle)

        #expect(ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .mouseRight))
        #expect(ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .shortcut))
    }

    @Test func movingLeftClickIsNotFlagged() {
        let buttons = Self.buttons(left: .mouseLeft, side1: .mouseLeft)

        #expect(!ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .mouseRight))
    }

    @Test func keepingLeftClickIsNotFlagged() {
        let buttons = Self.buttons(left: .mouseLeft, side1: .dpiCycle)

        #expect(!ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .mouseLeft))
    }

    @Test func changingAnotherButtonIsNotFlagged() {
        let buttons = Self.buttons(left: .mouseLeft, side1: .dpiCycle)

        #expect(!ButtonRules.losesPrimaryClick(buttons: buttons, button: .side1, action: .disabled))
    }

    @Test func otherActionIsNotALeftClick() {
        let other = Action.other(class: 9, value1: 0, value2: 0)
        let buttons = Self.buttons(left: other, side1: .dpiCycle)

        #expect(!ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .mouseRight))
    }

    @Test func shortcutIsNotALeftClick() {
        let buttons = Self.buttons(left: .shortcut, side1: .dpiCycle)

        #expect(!ButtonRules.losesPrimaryClick(buttons: buttons, button: .left, action: .mouseRight))
    }

}
