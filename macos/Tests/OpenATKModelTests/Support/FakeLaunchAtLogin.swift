//
//  FakeLaunchAtLogin.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import ServiceManagement

/// Stands in for `SMAppService.mainApp`, so no test registers a login item.
final class FakeLaunchAtLogin: LaunchAtLogin {

    struct Failure: Error {}

    var status: SMAppService.Status = .notRegistered

    /// The status a successful `register` leaves.
    var statusAfterRegister: SMAppService.Status = .enabled

    /// Whether `register` and `unregister` throw.
    var fails = false

    private(set) var calls: [String] = []

    func register() throws {
        calls.append("register")

        if fails {
            throw Failure()
        }

        status = statusAfterRegister
    }

    func unregister() throws {
        calls.append("unregister")

        if fails {
            throw Failure()
        }

        status = .notRegistered
    }

}
