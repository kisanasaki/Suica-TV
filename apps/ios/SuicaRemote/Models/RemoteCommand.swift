//
// Suica Core Remote API v1へ送るコマンドとparamsを定義する。
// rawValueとJSON fieldはWire契約のため、Core側と互換性を維持する。
//

import Foundation

/// Coreと共有する表示モード。rawValueはWire契約の一部である。
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

/// Remote API v1で受け付ける操作名。rawValueの変更は後方互換性を壊す。
enum RemoteAction: String, Codable, CaseIterable, Sendable {
    case up = "navigation.up"
    case down = "navigation.down"
    case left = "navigation.left"
    case right = "navigation.right"
    case select = "navigation.select"
    case back = "navigation.back"
    case home = "navigation.home"
    case switchMode = "system.switch_mode"
    case scroll = "pointer.scroll"
    case pointerMove = "pointer.move"
    case pointerClick = "pointer.click"
    case inputText = "input.text"
    case deleteBackward = "input.delete_backward"
    case submitText = "input.submit"
}

struct CommandParams: Encodable, Equatable, Sendable {
    let mode: DisplayMode?
    let text: String?
    let dx: Int?
    let dy: Int?

    init(mode: DisplayMode) {
        self.mode = mode
        text = nil
        dx = nil
        dy = nil
    }

    init(text: String) {
        mode = nil
        self.text = text
        dx = nil
        dy = nil
    }

    init(dx: Int, dy: Int) {
        mode = nil
        text = nil
        self.dx = dx
        self.dy = dy
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encodeIfPresent(mode, forKey: .mode)
        try container.encodeIfPresent(text, forKey: .text)
        try container.encodeIfPresent(dx, forKey: .dx)
        try container.encodeIfPresent(dy, forKey: .dy)
    }

    private enum CodingKeys: String, CodingKey {
        case mode
        case text
        case dx
        case dy
    }
}

/// 1回のremote.command要求。requestIdで非同期の結果と対応付ける。
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
