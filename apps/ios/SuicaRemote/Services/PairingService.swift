//
// 6桁コードと端末名をCoreのペアリングAPIへ送信する。
// HTTP処理とdecodeだけを担当し、取得したトークンの保存はViewModelへ委譲する。
//

import Foundation

struct PairingResponse: Decodable, Equatable, Sendable {
    let deviceId: UUID
    let token: String
}

/// CoreのペアリングHTTP APIを呼び出す境界。
protocol PairingServiceProtocol: Sendable {
    func pair(configuration: ConnectionConfiguration, code: String, deviceName: String) async throws -> PairingResponse
}

/// HTTP statusとresponse bodyをアプリ共通のエラーへ変換する実装。
struct PairingService: PairingServiceProtocol {
    private struct RequestBody: Encodable {
        let code: String
        let deviceName: String
    }

    private struct ErrorEnvelope: Decodable {
        let error: ServerError
    }

    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func pair(
        configuration: ConnectionConfiguration,
        code: String,
        deviceName: String
    ) async throws -> PairingResponse {
        guard let url = configuration.pairingURL else { throw RemoteClientError.invalidURL }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.timeoutInterval = 15
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(RequestBody(code: code, deviceName: deviceName))

        do {
            let (data, response) = try await session.data(for: request)
            guard let http = response as? HTTPURLResponse else {
                throw RemoteClientError.invalidPayload
            }
            guard http.statusCode == 201 else {
                if http.statusCode == 401 { throw RemoteClientError.unauthorized }
                if let envelope = try? JSONDecoder().decode(ErrorEnvelope.self, from: data) {
                    throw RemoteClientError.transport(envelope.error.message)
                }
                throw RemoteClientError.transport(String(localized: "ペアリングに失敗しました。"))
            }
            return try JSONDecoder().decode(PairingResponse.self, from: data)
        } catch let error as RemoteClientError {
            throw error
        } catch is DecodingError {
            throw RemoteClientError.invalidPayload
        } catch {
            throw RemoteClientError.transport(error.localizedDescription)
        }
    }
}
