import XCTest
@testable import SuicaRemote

actor FakeWebSocketClient: WebSocketClientProtocol {
    private let stream: AsyncThrowingStream<ServerMessage, Error>
    private let continuation: AsyncThrowingStream<ServerMessage, Error>.Continuation
    private(set) var sentCommands: [RemoteCommand] = []

    init() {
        (stream, continuation) = AsyncThrowingStream.makeStream()
    }

    func connect(configuration: ConnectionConfiguration) async throws { }
    func disconnect() async { }
    func send(_ command: RemoteCommand) async throws { sentCommands.append(command) }
    func messages() -> AsyncThrowingStream<ServerMessage, Error> { stream }
    func emit(_ message: ServerMessage) { continuation.yield(message) }
    func sentCount() -> Int { sentCommands.count }
}

actor DelayedWebSocketClient: WebSocketClientProtocol {
    private(set) var connectCount = 0
    private(set) var disconnectCount = 0

    func connect(configuration: ConnectionConfiguration) async throws {
        connectCount += 1
    }

    func disconnect() async {
        disconnectCount += 1
        try? await Task.sleep(for: .milliseconds(20))
    }

    func send(_ command: RemoteCommand) async throws { }

    func messages() -> AsyncThrowingStream<ServerMessage, Error> {
        AsyncThrowingStream { _ in }
    }

    func counts() -> (connects: Int, disconnects: Int) {
        (connectCount, disconnectCount)
    }
}

actor RecoveringWebSocketClient: WebSocketClientProtocol {
    private(set) var connectCount = 0
    private var streamCount = 0

    func connect(configuration: ConnectionConfiguration) async throws {
        connectCount += 1
    }

    func disconnect() async { }
    func send(_ command: RemoteCommand) async throws { }

    func messages() -> AsyncThrowingStream<ServerMessage, Error> {
        streamCount += 1
        if streamCount == 1 {
            return AsyncThrowingStream { continuation in
                continuation.finish(throwing: RemoteClientError.disconnected)
            }
        }
        return AsyncThrowingStream { continuation in
            continuation.yield(.hello(
                protocolVersion: 1,
                serverVersion: "0.1.0",
                connectionId: UUID(),
                role: "remote"
            ))
        }
    }

    func connections() -> Int { connectCount }
}

final class MemoryKeychain: KeychainStoreProtocol, @unchecked Sendable {
    var token: String?
    init(token: String? = "test-token") { self.token = token }
    func readToken() throws -> String? { token }
    func saveToken(_ token: String) throws { self.token = token }
    func deleteToken() throws { token = nil }
}

struct FixedSettingsStore: AppSettingsStoreProtocol {
    func loadHost() -> String { "raspberrypi.local" }
    func loadPort() -> Int { 3030 }
    func save(host: String, port: Int) { }
}

struct UnusedPairingService: PairingServiceProtocol {
    func pair(configuration: ConnectionConfiguration, code: String, deviceName: String) async throws -> PairingResponse {
        PairingResponse(deviceId: UUID(), token: "paired-token")
    }
}

@MainActor
final class RemoteViewModelTests: XCTestCase {
    func testDoesNotSendWhileDisconnected() async {
        let client = FakeWebSocketClient()
        let viewModel = makeViewModel(client: client)
        viewModel.sendNavigation(.up)
        await Task.yield()
        let sentCount = await client.sentCount()
        XCTAssertEqual(sentCount, 0)
    }

    func testConnectsAfterHelloAndUsesServerState() async throws {
        let client = FakeWebSocketClient()
        let viewModel = makeViewModel(client: client)
        viewModel.start()
        await Task.yield()
        await client.emit(.hello(protocolVersion: 1, serverVersion: "0.1.0", connectionId: UUID(), role: "remote"))
        await client.emit(.systemState(mode: .pc, transitioning: false, targetMode: nil, changedAt: Date()))
        for _ in 0..<10 { await Task.yield() }
        XCTAssertEqual(viewModel.connectionState, .connected)
        XCTAssertEqual(viewModel.displayMode, .pc)

        viewModel.sendNavigation(.home)
        for _ in 0..<5 { await Task.yield() }
        let sentCount = await client.sentCount()
        XCTAssertEqual(sentCount, 1)
        viewModel.stop()
    }

    func testSendsFinalizedUnicodeTextAndClearsDraft() async throws {
        let client = FakeWebSocketClient()
        let viewModel = makeViewModel(client: client)
        viewModel.start()
        await Task.yield()
        await client.emit(.hello(protocolVersion: 1, serverVersion: "0.1.0", connectionId: UUID(), role: "remote"))
        for _ in 0..<5 { await Task.yield() }

        viewModel.textInput = "日本語🍉"
        viewModel.sendTextInput()
        for _ in 0..<5 { await Task.yield() }

        let commands = await client.sentCommands
        XCTAssertEqual(commands.last?.action, .inputText)
        XCTAssertEqual(commands.last?.params, CommandParams(text: "日本語🍉"))
        XCTAssertEqual(viewModel.textInput, "")
        viewModel.stop()
    }

    func testRejectsTextLongerThanTwoHundredUnicodeScalars() async {
        let client = FakeWebSocketClient()
        let viewModel = makeViewModel(client: client)
        viewModel.start()
        await Task.yield()
        await client.emit(.hello(protocolVersion: 1, serverVersion: "0.1.0", connectionId: UUID(), role: "remote"))
        for _ in 0..<5 { await Task.yield() }

        viewModel.textInput = String(repeating: "あ", count: 201)
        viewModel.sendTextInput()
        await Task.yield()

        let sentCount = await client.sentCount()
        XCTAssertEqual(sentCount, 0)
        XCTAssertNotNil(viewModel.alertMessage)
        viewModel.stop()
    }

    func testRapidReconnectRequestsCreateOnlyLatestConnectionLoop() async throws {
        let client = DelayedWebSocketClient()
        let viewModel = RemoteViewModel(
            webSocket: client,
            pairingService: UnusedPairingService(),
            keychain: MemoryKeychain(),
            settingsStore: FixedSettingsStore()
        )

        viewModel.retry()
        viewModel.retry()
        viewModel.retry()
        try await Task.sleep(for: .milliseconds(100))

        let counts = await client.counts()
        XCTAssertEqual(counts.connects, 1)
        viewModel.stop()
    }

    func testStopInvalidatesReconnectWaitingForDisconnect() async throws {
        let client = DelayedWebSocketClient()
        let viewModel = RemoteViewModel(
            webSocket: client,
            pairingService: UnusedPairingService(),
            keychain: MemoryKeychain(),
            settingsStore: FixedSettingsStore()
        )

        viewModel.retry()
        viewModel.stop()
        try await Task.sleep(for: .milliseconds(100))

        let counts = await client.counts()
        XCTAssertEqual(counts.connects, 0)
    }

    func testReconnectsAfterTransportDisconnectAndAcceptsNewHello() async throws {
        let client = RecoveringWebSocketClient()
        let viewModel = RemoteViewModel(
            webSocket: client,
            pairingService: UnusedPairingService(),
            keychain: MemoryKeychain(),
            settingsStore: FixedSettingsStore(),
            reconnectPolicy: ReconnectPolicy(delays: [0])
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(700))

        let connections = await client.connections()
        XCTAssertEqual(connections, 2)
        XCTAssertEqual(viewModel.connectionState, .connected)
        viewModel.stop()
    }

    private func makeViewModel(client: FakeWebSocketClient) -> RemoteViewModel {
        RemoteViewModel(
            webSocket: client,
            pairingService: UnusedPairingService(),
            keychain: MemoryKeychain(),
            settingsStore: FixedSettingsStore()
        )
    }
}
