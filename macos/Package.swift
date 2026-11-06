// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "openatk",
    platforms: [.macOS(.v15)],
    targets: [
        .binaryTarget(
            name: "OpenATKFFI",
            path: "Frameworks/OpenATKFFI.xcframework"
        ),
        .target(
            name: "OpenATKBindings",
            dependencies: ["OpenATKFFI"],
            linkerSettings: [
                .linkedFramework("IOKit"),
                .linkedFramework("CoreFoundation"),
            ]
        ),
        .target(
            name: "OpenATKModel",
            dependencies: ["OpenATKBindings"]
        ),
        .executableTarget(
            name: "OpenATK",
            dependencies: ["OpenATKBindings", "OpenATKModel"]
        ),
        .testTarget(
            name: "OpenATKModelTests",
            dependencies: ["OpenATKBindings", "OpenATKModel"]
        ),
    ]
)
