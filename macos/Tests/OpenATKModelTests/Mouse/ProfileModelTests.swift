//
//  ProfileModelTests.swift
//  OpenATKModelTests
//
//  Created by Yesset Murat on 06.11.2026.
//

import OpenATKBindings
import OpenATKModel
import Testing

@MainActor
struct ProfileModelTests {

    private let sink = RecordingSink()
    private let model: MouseModel

    init() {
        model = MouseModel(sink: sink)
        model.apply(.snapshot(snapshot: .sample()))
    }

    private static func profileSubmenu(_ snapshot: SnapshotValue) -> (String, [MenuChoice])? {
        MenuContent.entries(snapshot: snapshot, strings: Strings.self).compactMap { entry in
            if case .submenu(let title, let items) = entry,
               case .profile = items.first?.target {
                return (title, items)
            }

            return nil
        }.first
    }

    @Test func pickProfileSendsSetProfile() {
        model.selectProfile(index: 3)

        #expect(sink.commands == [.setProfile(index: 3)])
        #expect(model.isPending)
    }

    @Test func menuPickSendsSetProfile() throws {
        let snapshot = try #require(model.snapshot)
        let items = try #require(Self.profileSubmenu(snapshot)).1
        let choice = try #require(items.first { $0.target == .profile(index: 2) })

        model.pick(choice)

        #expect(sink.commands == [.setProfile(index: 2)])
    }

    @Test func sameProfileSendsNothing() {
        model.selectProfile(index: 1)

        #expect(sink.commands.isEmpty)
        #expect(!model.isPending)
    }

    @Test func samePickAfterFailureSendsSetProfile() {
        model.apply(.commandFailed(message: "timeout"))

        model.selectProfile(index: 1)

        #expect(sink.commands == [.setProfile(index: 1)])
        #expect(model.errorText == nil)
    }

    @Test func offerAboveTheCountStaysUnsentAfterFailure() {
        model.apply(.snapshot(snapshot: .sample(profile: 7)))
        model.apply(.commandFailed(message: "timeout"))

        model.selectProfile(index: 7)

        #expect(sink.commands.isEmpty)
    }

    @Test func indexNotBelowTheCountSendsNothing() {
        model.selectProfile(index: 4)

        #expect(sink.commands.isEmpty)
    }

    @Test func pickDuringPendingSendsNothing() {
        model.selectProfile(index: 2)

        model.selectProfile(index: 3)

        #expect(sink.commands == [.setProfile(index: 2)])
        #expect(model.snapshot?.profile == 1)
    }

    @Test func snapshotEndsTheProfileWrite() {
        model.selectProfile(index: 2)

        model.apply(.snapshot(snapshot: .sample(profile: 2)))

        #expect(!model.isPending)
        #expect(model.snapshot?.profile == 2)
    }

    @Test func profileChangeClearsDraft() {
        model.editField(.stage(0), text: "999")

        model.apply(.snapshot(snapshot: .sample(profile: 2, stages: [500, 800, 1200, 1600])))

        #expect(model.fieldText(.stage(0)) == "500")
    }

    @Test func profileActionsOffered() {
        let choices = model.actionChoices

        #expect(choices.suffix(6) == [
            .profile(index: 0),
            .profile(index: 1),
            .profile(index: 2),
            .profile(index: 3),
            .profileCycle,
            .shortcut,
        ])
        #expect(Strings.action(.profile(index: 0)) == "Profile 1")
        #expect(Strings.action(.profileCycle) == "Cycle profile")
    }

    @Test func profileActionsFollowTheCount() {
        var snapshot = SnapshotValue.sample()
        snapshot.profileCount = 2

        model.apply(.snapshot(snapshot: snapshot))

        #expect(!model.actionChoices.contains(.profile(index: 2)))
        #expect(model.actionChoices.contains(.profile(index: 1)))
    }

    @Test func profileChoicesListTheCount() {
        let choices = SnapshotValue.sample().profileChoices

        #expect(choices.map(\.value) == [0, 1, 2, 3])
        #expect(choices.map(\.isSelectable) == [true, true, true, true])
    }

    @Test(arguments: [(UInt8(4), "Profile: 5", "Profile 5"), (255, "Profile: 256", "Profile 256")])
    func profileAboveCountShowsItself(profile: UInt8, title: String, itemTitle: String) throws {
        let snapshot = SnapshotValue.sample(profile: profile)

        let picker = snapshot.profileChoices
        let (menuTitle, items) = try #require(Self.profileSubmenu(snapshot))
        let last = try #require(items.last)

        #expect(picker.last == PickerChoice(value: profile, isSelectable: false))
        #expect(picker.count == 5)
        #expect(menuTitle == title)
        #expect(items.count == 5)
        #expect(last.title == itemTitle)
        #expect(last.isChecked)
        #expect(!last.isEnabled)
    }

    @Test func profileAboveCountCannotBeWrittenButListedOnesCan() {
        model.apply(.snapshot(snapshot: .sample(profile: 7)))

        model.selectProfile(index: 7)
        model.selectProfile(index: 0)

        #expect(sink.commands == [.setProfile(index: 0)])
    }

}
