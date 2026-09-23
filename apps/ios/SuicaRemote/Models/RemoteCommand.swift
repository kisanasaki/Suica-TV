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
    case inputText = "input.text"
    case deleteBackward = "input.delete_backward"
    case submitText = "input.submit"
}

struct CommandParams: Encodable, Equatable, Sendable {
    let mode: DisplayMode?
    let text: String?

    init(mode: DisplayMode) {
        self.mode = mode
        text = nil
    }

    init(text: String) {
        mode = nil
        self.text = text
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encodeIfPresent(mode, forKey: .mode)
        try container.encodeIfPresent(text, forKey: .text)
    }

    private enum CodingKeys: String, CodingKey {
        case mode
        case text
    }
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
