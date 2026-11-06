//
//  MenuContent.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// What the menu bar item shows, worked out from the snapshot alone.
public enum MenuContent {

    /// The text beside the menu bar icon: the battery, or a dash when no mouse can be reached.
    public static func title(snapshot: SnapshotValue?) -> String {
        guard let snapshot else {
            return Strings.noValue
        }

        return Strings.batteryPercent(snapshot.percent)
    }

    /// The lines of the menu, in order. Without a mouse only opening the window and quitting
    /// remain.
    public static func entries(snapshot: SnapshotValue?, strings: Strings.Type) -> [MenuEntry] {
        let appEntries: [MenuEntry] = [.openWindow(strings.openFullWindow), .quit(strings.quit)]

        guard let snapshot else {
            return appEntries
        }

        let battery = MenuEntry.info(strings.battery(percent: snapshot.percent))

        return [battery, .separator]
            + stages(snapshot, strings: strings)
            + [.separator]
            + rates(snapshot, strings: strings)
            + [.separator]
            + appEntries
    }

    private static func stages(_ snapshot: SnapshotValue, strings: Strings.Type) -> [MenuEntry] {
        snapshot.stages.enumerated().map { index, stage in
            let title = stage.x == stage.y
                ? strings.dpi(stage.x)
                : strings.splitDpi(x: stage.x, y: stage.y)

            return .choice(
                MenuChoice(
                    title: title,
                    target: .stage(index: index),
                    isChecked: index == Int(snapshot.currentStage)
                )
            )
        }
    }

    /// The rates on offer, then the mouse's own if it is not one of them: it can be seen as the
    /// current value but not picked.
    private static func rates(_ snapshot: SnapshotValue, strings: Strings.Type) -> [MenuEntry] {
        var hertzValues = snapshot.pollingOptionsHz

        if !hertzValues.contains(snapshot.pollingHz) {
            hertzValues.append(snapshot.pollingHz)
        }

        return hertzValues.map { hertz in
            .choice(
                MenuChoice(
                    title: strings.hertz(hertz),
                    target: .rate(hertz: hertz),
                    isChecked: hertz == snapshot.pollingHz,
                    isEnabled: snapshot.pollingOptionsHz.contains(hertz)
                )
            )
        }
    }

}
