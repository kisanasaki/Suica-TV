import Foundation

struct ConnectionConfiguration: Equatable, Sendable {
    let host: String
    let port: Int
    let token: String?

    var webSocketURL: URL? {
        var components = URLComponents()
        components.scheme = "ws"
        components.host = host
        components.port = port
        components.path = "/ws"
        components.queryItems = [
            URLQueryItem(name: "role", value: "remote"),
            URLQueryItem(name: "protocolVersion", value: "1")
        ]
        return components.url
    }

    var pairingURL: URL? {
        var components = URLComponents()
        components.scheme = "http"
        components.host = host
        components.port = port
        components.path = "/api/v1/pair"
        return components.url
    }
}

enum SettingsValidationError: LocalizedError, Equatable {
    case invalidHost
    case invalidPort
    case invalidPairingCode

    var errorDescription: String? {
        switch self {
        case .invalidHost: String(localized: "ホスト名またはIPv4アドレスを入力してください。")
        case .invalidPort: String(localized: "ポートは1から65535の範囲で入力してください。")
        case .invalidPairingCode: String(localized: "6桁のペアリングコードを入力してください。")
        }
    }
}

enum SettingsValidator {
    static func validatedHost(_ value: String) throws -> String {
        let host = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !host.isEmpty,
              !host.contains("://"),
              !host.contains("/"),
              !host.contains("?"),
              !host.contains("#"),
              !host.contains(where: { $0.isWhitespace }),
              isIPv4(host) || isHostname(host)
        else { throw SettingsValidationError.invalidHost }
        return host
    }

    static func validatedPort(_ value: String) throws -> Int {
        guard let port = Int(value), (1...65_535).contains(port) else {
            throw SettingsValidationError.invalidPort
        }
        return port
    }

    static func validatedPairingCode(_ value: String) throws -> String {
        let code = value.trimmingCharacters(in: .whitespacesAndNewlines)
        guard code.count == 6, code.allSatisfy(\.isNumber) else {
            throw SettingsValidationError.invalidPairingCode
        }
        return code
    }

    private static func isIPv4(_ value: String) -> Bool {
        let parts = value.split(separator: ".", omittingEmptySubsequences: false)
        return parts.count == 4 && parts.allSatisfy { part in
            guard !part.isEmpty, part.count <= 3, let number = Int(part) else { return false }
            return (0...255).contains(number)
        }
    }

    private static func isHostname(_ value: String) -> Bool {
        guard value.count <= 253 else { return false }
        return value.split(separator: ".", omittingEmptySubsequences: false).allSatisfy { label in
            guard !label.isEmpty, label.count <= 63,
                  label.first?.isLetter == true || label.first?.isNumber == true,
                  label.last?.isLetter == true || label.last?.isNumber == true
            else { return false }
            return label.allSatisfy { $0.isLetter || $0.isNumber || $0 == "-" }
        }
    }
}

protocol AppSettingsStoreProtocol: Sendable {
    func loadHost() -> String
    func loadPort() -> Int
    func save(host: String, port: Int)
}

struct AppSettingsStore: AppSettingsStoreProtocol, @unchecked Sendable {
    private let defaults: UserDefaults
    private let hostKey = "suicaRemote.host"
    private let portKey = "suicaRemote.port"

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func loadHost() -> String {
        defaults.string(forKey: hostKey) ?? "raspberrypi.local"
    }

    func loadPort() -> Int {
        let stored = defaults.integer(forKey: portKey)
        return stored == 0 ? 3030 : stored
    }

    func save(host: String, port: Int) {
        defaults.set(host, forKey: hostKey)
        defaults.set(port, forKey: portKey)
    }
}
