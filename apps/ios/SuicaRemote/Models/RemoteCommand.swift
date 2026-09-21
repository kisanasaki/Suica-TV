import Foundation

enum DisplayMode: String, Codable, CaseIterable, Sendable {
    case tv
    case pc

    var localizedName: String {
        switch self {
        case .tv: String(localized: "TVモード")
        case .pc: String(localized: "PCモード")
        }
    }
}

enum RemoteAction: String, Codable, CaseIterable, Sendable {
    case up = "navigation.up"
    case down = "navigation.down"
    case left = "navigation.left"
    case right = "navigation.right"
    case select = "navigation.select"
    case back = "navigation.back"
    case home = "navigation.home"
    case switchMode = "system.switch_mode"
}

struct CommandParams: Encodable, Equatable, Sendable {
    let mode: DisplayMode
}

struct RemoteCommand: Encodable, Equatable, Sendable {
    let type = "remote.command"
    let requestId: UUID
    let action: RemoteAction
    let params: CommandParams?

    init(requestId: UUID = UUID(), action: RemoteAction, params: CommandParams? = nil) {
        self.requestId = requestId
        self.action = action
        self.params = params
    }
}
