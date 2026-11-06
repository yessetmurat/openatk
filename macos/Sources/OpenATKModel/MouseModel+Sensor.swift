//
//  MouseModel+Sensor.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import Foundation
import OpenATKBindings

extension MouseModel {

    /// The lowest debounce time that does not need a warning, in milliseconds.
    private static let safeDebounceMs: UInt8 = 4

    /// The entries of the lift-off picker, in tenths of a millimetre.
    public var liftOffChoices: [PickerChoice<UInt8>] {
        guard let snapshot else {
            return []
        }

        let options = stride(
            from: snapshot.liftOffMinTenths,
            through: snapshot.liftOffMaxTenths,
            by: 1
        )

        return PickerChoice.list(options: Array(options), current: snapshot.liftOffTenths)
    }

    /// The entries of the debounce picker, in milliseconds.
    public var debounceChoices: [PickerChoice<UInt8>] {
        guard let snapshot else {
            return []
        }

        return PickerChoice.list(
            options: Array(snapshot.debounceOptionsMs),
            current: snapshot.debounceMs
        )
    }

    /// The entries of the sleep picker, in seconds.
    public var sleepChoices: [PickerChoice<UInt16>] {
        guard let snapshot else {
            return []
        }

        return PickerChoice.list(
            options: snapshot.sleepOptionsSeconds,
            current: snapshot.sleepSeconds
        )
    }

    /// Whether the debounce time is low enough to warn about.
    public var showsDebounceWarning: Bool {
        guard let debounceMs = snapshot?.debounceMs else {
            return false
        }

        return debounceMs < Self.safeDebounceMs
    }

    /// Whether the mouse has a sensor angle.
    public var showsAngle: Bool {
        snapshot?.angle != nil
    }

    /// Sets the lift-off height to a listed value, unless it is the current one.
    public func selectLiftOff(tenths: UInt8) {
        select(
            tenths,
            current: snapshot?.liftOffTenths,
            choices: liftOffChoices,
            command: .setLiftOff(tenths: tenths)
        )
    }

    /// Sets the debounce time to a listed value, unless it is the current one.
    public func selectDebounce(ms: UInt8) {
        select(
            ms,
            current: snapshot?.debounceMs,
            choices: debounceChoices,
            command: .setDebounce(ms: ms)
        )
    }

    /// Sets the sleep time to a listed value.
    ///
    /// The current value goes out again only after a failure. One pick makes several writes and
    /// the snapshot shows only the first, so writing it again is the only way to heal a pick that
    /// was half applied.
    public func selectSleep(seconds: UInt16) {
        guard let current = snapshot?.sleepSeconds,
              seconds != current || errorText != nil,
              sleepChoices.contains(PickerChoice(value: seconds, isSelectable: true)) else {
            return
        }

        send(.setSleep(seconds: seconds))
    }

    /// Turns motion sync on or off, unless it already is.
    public func selectMotionSync(enabled: Bool) {
        toggle(snapshot?.motionSync, to: enabled, command: .setMotionSync(enabled: enabled))
    }

    /// Turns linear correction on or off, unless it already is.
    public func selectLinearCorrection(enabled: Bool) {
        toggle(
            snapshot?.linearCorrection,
            to: enabled,
            command: .setLinearCorrection(enabled: enabled)
        )
    }

    /// Turns ripple control on or off, unless it already is.
    public func selectRippleControl(enabled: Bool) {
        toggle(snapshot?.rippleControl, to: enabled, command: .setRippleControl(enabled: enabled))
    }

    /// Turns far distance mode on or off, unless it already is.
    public func selectFarDistance(enabled: Bool) {
        toggle(snapshot?.farDistance, to: enabled, command: .setFarDistance(enabled: enabled))
    }

    /// Turns the sensor angle on or off, unless it already is.
    public func selectAngleEnabled(_ enabled: Bool) {
        toggle(snapshot?.angle?.enabled, to: enabled, command: .setAngleEnabled(enabled: enabled))
    }

    /// The write for text typed into the angle field, or `nil` when the text is not a whole number
    /// in the snapshot's range, or is the shown value.
    func angleCommand(text: String) -> Command? {
        guard let snapshot,
              let degrees = snapshot.angle?.degrees,
              let typed = Int8(text),
              typed != degrees,
              typed >= snapshot.angleMin,
              typed <= snapshot.angleMax else {
            return nil
        }

        return .setAngle(degrees: typed)
    }

    private func select<Value: Hashable>(
        _ value: Value,
        current: Value?,
        choices: [PickerChoice<Value>],
        command: Command
    ) {
        guard let current,
              value != current,
              choices.contains(PickerChoice(value: value, isSelectable: true)) else {
            return
        }

        send(command)
    }

    private func toggle(_ current: Bool?, to enabled: Bool, command: Command) {
        guard let current,
              current != enabled else {
            return
        }

        send(command)
    }

}
