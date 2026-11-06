//
//  MainWindow.swift
//  OpenATK
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKModel
import SwiftUI

/// The window: a fixed sidebar, and a detail with the header, the section and the error line.
struct MainWindow: View {

    /// The identifier of the window's scene.
    static let sceneID = "main"

    private static let size = CGSize(width: 800, height: 540)
    private static let sidebarWidth: CGFloat = 220
    private static let fontFamily = "IBM Plex Sans"
    private static let fontSize: CGFloat = 13
    private static let sidebarFontSize: CGFloat = 13

    let model: MouseModel
    let preferences: Preferences
    let launchAtLogin: LaunchAtLoginSetting

    @Environment(\.scenePhase) private var scenePhase

    @State private var section = SidebarSection.performance

    var body: some View {
        // `NavigationSplitView` gives the sidebar the system's Liquid Glass look.
        NavigationSplitView(columnVisibility: .constant(.all)) {
            sidebar
        } detail: {
            detail
        }
        .frame(width: Self.size.width, height: Self.size.height)
        // Observed with `MenuBarExtra` present: closing only hides the window, so its sections get
        // no `onDisappear`. Minimizing gives `.inactive`, and must not leave the key monitor on.
        .onChange(of: scenePhase) {
            if scenePhase != .active {
                model.cancelCapture()
            }

            if scenePhase == .background {
                model.discardFieldEdit()
            }
        }
    }

    private var sidebar: some View {
        List(selection: $section) {
            ForEach(SidebarSection.allCases, id: \.self) { section in
                Label(section.title, systemImage: section.symbol)
                    .font(.custom(Self.fontFamily, size: Self.sidebarFontSize))
            }
        }
        .labelStyle(SidebarLabelStyle())
        .environment(\.sidebarRowSize, .large)
        .toolbar(removing: .sidebarToggle)
        // Must stay last: a modifier after it hides the width from the split view.
        .navigationSplitViewColumnWidth(Self.sidebarWidth)
    }

    private var detail: some View {
        VStack(spacing: 0) {
            Header(model: model)
            Divider()
            sectionView
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            errorLine
        }
        // The title bar is hidden, so nothing needs the space it reserves.
        .ignoresSafeArea(.container, edges: .top)
        .navigationTitle(section.title)
        .toolbar(removing: .title)
        .toolbarBackgroundVisibility(.hidden, for: .windowToolbar)
        // Not on the root: a sidebar label whose font equals an inherited one loses it to the
        // system's row font, so the sidebar size could not match this one.
        .font(.custom(Self.fontFamily, size: Self.fontSize))
    }

    @ViewBuilder private var sectionView: some View {
        switch section {
        case .performance:
            PerformanceView(model: model)
        case .buttons:
            ButtonsView(model: model)
        case .sensor:
            SensorView(model: model)
        case .settings:
            SettingsView(model: model, preferences: preferences, launchAtLogin: launchAtLogin)
        }
    }

    @ViewBuilder private var errorLine: some View {
        if let errorText = model.errorText ?? launchAtLogin.errorText {
            Divider()
            Label(errorText, systemImage: "exclamationmark.triangle.fill")
                .foregroundStyle(.red)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 20)
                .padding(.vertical, 10)
        }
    }

}
