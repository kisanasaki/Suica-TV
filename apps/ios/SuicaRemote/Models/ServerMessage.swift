//
// Suica Coreから受信するserver messageを型安全にdecodeする。
// 未知messageは互換性のため保持しつつ、既知messageの必須値は厳密に検証する。
//

import Foundation

struct ServerError: Codable, Equatable, Sendable {
    let code: String
    let message: String
    let retryable: Bool
}

/// Coreから届くmessageの判別共用体。未知typeは接続を切らずに保持する。
enum ServerMessage: Decodable, Equatable, Sendable {
    case hello(protocolVersion: Int, serverVersion: String, connectionId: UUID, role: String)
    case systemState(mode: DisplayMode, transitioning: Bool, targetMode: DisplayMode?, changedAt: Date)
    case commandResult(requestId: UUID, ok: Bool, error: ServerError?)
    case error(ServerError)
    case shutdown(retryAfterSeconds: Int)
    case unknown(type: String)

    private enum CodingKeys: String, CodingKey {
        case type
        case protocolVersion
        case serverVersion
        case connectionId
        case role
        case mode
        case transitioning
        case targetMode
        case changedAt
        case requestId
        case ok
        case error
        case retryAfterSeconds
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "server.hello":
            self = try .hello(
                protocolVersion: container.decode(Int.self, forKey: .protocolVersion),
                serverVersion: container.decode(String.self, forKey: .serverVersion),
                connectionId: container.decode(UUID.self, forKey: .connectionId),
                role: container.decode(String.self, forKey: .role)
            )
        case "system.state":
            let timestamp = try container.decode(String.self, forKey: .changedAt)
            guard let changedAt = Self.parseRFC3339(timestamp) else {
                throw DecodingError.dataCorruptedError(
                    forKey: .changedAt,
                    in: container,
                    debugDescription: "changedAt must be an RFC 3339 timestamp"
                )
            }
            self = try .systemState(
                mode: container.decode(DisplayMode.self, forKey: .mode),
                transitioning: container.decode(Bool.self, forKey: .transitioning),
                targetMode: container.decodeIfPresent(DisplayMode.self, forKey: .targetMode),
                changedAt: changedAt
            )
        case "command.result":
            self = try .commandResult(
                requestId: container.decode(UUID.self, forKey: .requestId),
                ok: container.decode(Bool.self, forKey: .ok),
                error: container.decodeIfPresent(ServerError.self, forKey: .error)
            )
        case "error":
            self = try .error(container.decode(ServerError.self, forKey: .error))
        case "server.shutdown":
            self = try .shutdown(retryAfterSeconds: container.decode(Int.self, forKey: .retryAfterSeconds))
        default:
            self = .unknown(type: type)
        }
    }

    private static func parseRFC3339(_ value: String) -> Date? {
        let fractional = ISO8601DateFormatter()
        fractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return fractional.date(from: value) ?? ISO8601DateFormatter().date(from: value)
    }
}
