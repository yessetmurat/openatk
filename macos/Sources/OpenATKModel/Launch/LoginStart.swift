//
//  LoginStart.swift
//  OpenATKModel
//
//  Created by Yesset Murat on 06.11.2026.
//

import CoreServices

/// How to recognize a start at login.
public enum LoginStart {

    /// Whether the app's open event says it was started as a login item.
    ///
    /// - Parameters:
    ///   - eventID: The ID of the Apple event being handled, if any.
    ///   - launchKind: The event's `keyAEPropData` parameter as an enum code, if any.
    public static func isLoginStart(eventID: AEEventID?, launchKind: OSType?) -> Bool {
        eventID == AEEventID(kAEOpenApplication)
            && launchKind == OSType(keyAELaunchedAsLogInItem)
    }

}
