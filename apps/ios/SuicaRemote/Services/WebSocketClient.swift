import Foundation

enum RemoteClientError: LocalizedError, Equatable, Sendable {
    case missingToken
    case invalidURL
    case unauthorized
    case protocolMismatch
    case invalidPayload
    case disconnected
    case transport(String)

    var errorDescription: String? {
        switch self {
        case .missingToken, .unauthorized: String(localized: "再ペアリングが必要です。")
        case .invalidURL: String(localized: "接続先の設定が正しくありません。")
        case .protocolMismatch: String(localized: "アプリの更新が必要です。")
        case .invalidPayload: String(localized: "Suica TVから不正な応答を受信しました。")
        case .disconnected: String(localized: "Suica TVとの接続が切れました。")
        case let .transport(message): message
        }
    }
}

protocol WebSocketClientProtocol: Sendable {
    func connect(configuration: ConnectionConfiguration) async throws
    func disconnect() async
    func send(_ command: RemoteCommand) async throws
    func messages() async -> AsyncThrowingStream<ServerMessage, Error>
}

actor WebSocketClient: WebSocketClientProtocol {
    private let session: URLSession
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()
    private var socket: URLSessionWebSocketTask?
    private var receiveTask: Task<Void, Never>?

    init(session: URLSession = .shared) {
        self.session = session
    }

    func connect(configuration: ConnectionConfiguration) async throws {
        guard let token = configuration.token, !token.isEmpty else {
            throw RemoteClientError.missingToken
        }
        guard let url = configuration.webSocketURL else {
            throw RemoteClientError.invalidURL
        }

        await disconnect()
        var request = URLRequest(url: url)
        request.timeoutInterval = 15
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        let task = session.webSocketTask(with: request)
        socket = task
        task.resume()
    }

    func disconnect() async {
        receiveTask?.cancel()
        receiveTask = nil
        socket?.cancel(with: .normalClosure, reason: nil)
        socket = nil
    }

    func send(_ command: RemoteCommand) async throws {
        guard let socket else { throw RemoteClientError.disconnected }
        do {
            let data = try encoder.encode(command)
            guard let text = String(data: data, encoding: .utf8) else {
                throw RemoteClientError.invalidPayload
            }
            try await socket.send(.string(text))
        } catch let error as RemoteClientError {
            throw error
        } catch {
            throw map(error, response: socket.response)
        }
    }

    func messages() -> AsyncThrowingStream<ServerMessage, Error> {
        guard let socket else {
            return AsyncThrowingStream { $0.finish(throwing: RemoteClientError.disconnected) }
        }
        let (stream, continuation) = AsyncThrowingStream<ServerMessage, Error>.makeStream()
        receiveTask?.cancel()
        let task = Task { [weak self] in
            guard let self else { return }
            await self.receiveMessages(from: socket, continuation: continuation)
        }
        receiveTask = task
        continuation.onTermination = { _ in task.cancel() }
        return stream
    }

    private func receiveMessages(
        from socket: URLSessionWebSocketTask,
        continuation: AsyncThrowingStream<ServerMessage, Error>.Continuation
    ) async {
        do {
            while !Task.isCancelled {
                let frame = try await socket.receive()
                let data: Data
                switch frame {
                case let .string(text): data = Data(text.utf8)
                case let .data(value): data = value
                @unknown default: throw RemoteClientError.invalidPayload
                }
                continuation.yield(try decoder.decode(ServerMessage.self, from: data))
            }
            continuation.finish()
        } catch is CancellationError {
            continuation.finish()
        } catch {
            continuation.finish(throwing: map(error, response: socket.response))
        }
    }

    private func map(_ error: Error, response: URLResponse?) -> RemoteClientError {
        if let status = (response as? HTTPURLResponse)?.statusCode {
            if status == 401 { return .unauthorized }
            if status == 400 { return .protocolMismatch }
        }
        if let remoteError = error as? RemoteClientError { return remoteError }
        if let urlError = error as? URLError {
            if urlError.code == .userAuthenticationRequired { return .unauthorized }
            if urlError.code == .cancelled { return .disconnected }
            return .transport(urlError.localizedDescription)
        }
        return .transport(error.localizedDescription)
    }
}
