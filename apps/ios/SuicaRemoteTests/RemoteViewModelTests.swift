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

    private func makeViewModel(client: FakeWebSocketClient) -> RemoteViewModel {
        RemoteViewModel(
            webSocket: client,
            pairingService: UnusedPairingService(),
            keychain: MemoryKeychain(),
            settingsStore: FixedSettingsStore()
        )
    }
}
