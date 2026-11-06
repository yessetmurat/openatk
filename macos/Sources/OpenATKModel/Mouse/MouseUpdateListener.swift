//
//  MouseUpdateListener.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import Dispatch
import OpenATKBindings

/// Carries updates from the service thread to the model.
public final class MouseUpdateListener: MouseListener, Sendable {

    /// The model that receives updates. The client needs the listener before the model can
    /// exist. Set this in the same main-thread turn that creates the client: hopped updates
    /// run after that turn, so none is lost.
    @MainActor public var model: MouseModel?

    /// Creates a listener with no model.
    public init() {}

    /// Hops to the main thread. `DispatchQueue.main` is FIFO, which keeps a failure ahead of
    /// its snapshot; a `Task` on the main actor does not promise that order.
    public func onEvent(event: MouseUpdate) {
        DispatchQueue.main.async {
            self.model?.apply(event)
        }
    }

}
