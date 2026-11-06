//
//  OpenATKApp.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import SwiftUI

@main
struct OpenATKApp: App {

    private let model = OpenATKApp.makeModel()

    var body: some Scene {
        Window(Strings.appName, id: "main") {
            MainWindow(model: model)
        }
        .windowResizability(.contentSize)
    }

    /// Starts the mouse service. This is the one place that creates a `MouseClient`.
    private static func makeModel() -> MouseModel {
        let listener = MouseUpdateListener()
        let client = MouseClient(listener: listener)
        let model = MouseModel(sink: client)

        listener.model = model

        return model
    }

}
