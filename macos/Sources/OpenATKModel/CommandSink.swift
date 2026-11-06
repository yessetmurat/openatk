//
//  CommandSink.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings

/// Where the model sends commands.
public protocol CommandSink {

    /// Queues `command` for the mouse.
    func send(command: Command)

}

extension MouseClient: CommandSink {}
