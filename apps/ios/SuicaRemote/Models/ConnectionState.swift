//
// Remote画面へ公開する接続状態と表示用属性を定義する。
// URLSessionWebSocketTaskの詳細をUIへ漏らさず、利用者が判断できる状態へ正規化する。
//

import Foundation

/// transportの状態を、UI表示と操作可否に必要な粒度へ変換した状態。
enum ConnectionState: Equatable, Sendable {
    case disconnected
    case connecting
    case connected
    case reconnecting(attempt: Int)
    case failed(message: String)

    var isConnected: Bool {
        self == .connected
    }

    var label: String {
        switch self {
        case .disconnected: String(localized: "未接続")
        case .connecting: String(localized: "接続中")
        case .connected: String(localized: "接続済み")
        case let .reconnecting(attempt): String(localized: "再接続中（\(attempt)回目）")
        case .failed: String(localized: "接続失敗")
        }
    }

    var symbolName: String {
        switch self {
        case .connected: "checkmark.circle.fill"
        case .connecting, .reconnecting: "arrow.triangle.2.circlepath"
        case .disconnected: "wifi.slash"
        case .failed: "exclamationmark.triangle.fill"
        }
    }
}
