//
//  RecordingSink.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel

/// Records commands.
final class RecordingSink: CommandSink {

    private(set) var commands: [Command] = []

    func send(command: Command) {
        commands.append(command)
    }

}
