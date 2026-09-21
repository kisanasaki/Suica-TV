import Foundation

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
