//
//  LaunchAtLoginSetting.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import Observation
import ServiceManagement

/// The launch at login toggle. The system holds the state, so this keeps no value of its own: it
/// reads ``status`` again after every change, and a failed change shows the real state.
@MainActor
@Observable
public final class LaunchAtLoginSetting {

    /// The login item's state, as last read from the system.
    public private(set) var status: SMAppService.Status

    /// Counts the attempts to change the state. A toggle that applies it as its identity is rebuilt
    /// on each, so it shows the real state also when the attempt changed nothing: a failure, or a
    /// register that leaves the item waiting for approval.
    public private(set) var refreshCount = 0

    /// Why the last change failed. Kept apart from the mouse's error, which also decides whether
    /// the sleep timer can be written again.
    public private(set) var errorText: String?

    @ObservationIgnored private let service: any LaunchAtLogin

    /// Reads the state of `service`.
    public init(service: any LaunchAtLogin) {
        self.service = service
        status = service.status
    }

    /// Whether the app starts at login. An item waiting for approval does not.
    public var isEnabled: Bool {
        status == .enabled
    }

    /// Whether the user has to allow the item in System Settings.
    public var needsApproval: Bool {
        status == .requiresApproval
    }

    /// Reads the state again, as the user can change it in System Settings.
    public func refresh() {
        status = service.status
    }

    /// Turns launch at login on or off, unless it already is.
    public func set(enabled: Bool) {
        guard enabled != isEnabled else {
            return
        }

        do {
            if enabled {
                try service.register()
            } else {
                try service.unregister()
            }

            errorText = nil
        } catch {
            errorText = Strings.launchAtLoginFailed(reason: error.localizedDescription)
        }

        refresh()
        refreshCount += 1
    }

}
