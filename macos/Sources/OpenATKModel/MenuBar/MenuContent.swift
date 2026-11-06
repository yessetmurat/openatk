//
//  MenuContent.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// What the menu bar item shows, worked out from the snapshot alone.
public enum MenuContent {

    /// The text beside the menu bar icon: the battery, a dash when no mouse can be reached, or
    /// nothing while charging, when the bolt symbol stands in.
    public static func title(snapshot: SnapshotValue?) -> String {
        guard let snapshot else {
            return Strings.noValue
        }

        switch snapshot.batteryDisplay {
        case .charging:
            return ""
        case .percent(let percent):
            return Strings.batteryPercent(percent)
        }
    }

    /// Whether the menu bar item shows the bolt symbol in place of the battery text.
    public static func isCharging(snapshot: SnapshotValue?) -> Bool {
        snapshot?.batteryDisplay == .charging
    }

    /// The lines of the menu, in order. Without a mouse only opening the window and quitting
    /// remain.
    public static func entries(snapshot: SnapshotValue?, strings: Strings.Type) -> [MenuEntry] {
        let appEntries: [MenuEntry] = [.openWindow(strings.openFullWindow), .quit(strings.quit)]

        guard let snapshot else {
            return appEntries
        }

        let battery = MenuEntry.info(strings.battery(snapshot.batteryDisplay))

        return [
            battery,
            .separator,
            stages(snapshot, strings: strings),
            rates(snapshot, strings: strings),
            profiles(snapshot, strings: strings),
            .separator,
        ] + appEntries
    }

    private static func stages(_ snapshot: SnapshotValue, strings: Strings.Type) -> MenuEntry {
        let current = Int(snapshot.currentStage)
        let stage = snapshot.stages.indices.contains(current) ? snapshot.stages[current] : nil

        let items = snapshot.stages.enumerated().map { index, stage in
            MenuChoice(
                title: strings.dpi(stage),
                target: .stage(index: index),
                isChecked: index == current
            )
        }

        return .submenu(title: strings.dpiTitle(stage: stage), items: items)
    }

    /// The rates on offer, with the mouse's own among them if it is not one: it can be seen as the
    /// current value but not picked.
    private static func rates(_ snapshot: SnapshotValue, strings: Strings.Type) -> MenuEntry {
        let choices = PickerChoice.list(
            options: snapshot.pollingOptionsHz,
            current: snapshot.pollingHz
        )

        let items = choices.map { choice in
            MenuChoice(
                title: strings.hertz(choice.value),
                target: .rate(hertz: choice.value),
                isChecked: choice.value == snapshot.pollingHz,
                isEnabled: choice.isSelectable
            )
        }

        return .submenu(title: strings.pollingTitle(hertz: snapshot.pollingHz), items: items)
    }

    private static func profiles(_ snapshot: SnapshotValue, strings: Strings.Type) -> MenuEntry {
        let items = snapshot.profileChoices.map { choice in
            MenuChoice(
                title: strings.profile(index: choice.value),
                target: .profile(index: choice.value),
                isChecked: choice.value == snapshot.profile,
                isEnabled: choice.isSelectable
            )
        }

        return .submenu(title: strings.profileTitle(index: snapshot.profile), items: items)
    }

}
