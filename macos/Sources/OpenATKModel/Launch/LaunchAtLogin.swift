//
//  LaunchAtLogin.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import ServiceManagement

/// The app's login item, so tests can stand in for the system's.
public protocol LaunchAtLogin {

    /// The login item's state as the system reports it.
    var status: SMAppService.Status { get }

    /// Asks the system to start the app at login.
    func register() throws

    /// Asks the system to stop starting the app at login.
    func unregister() throws

}

extension SMAppService: LaunchAtLogin {}
