import Foundation
import Observation
import UIKit

@MainActor
@Observable
final class RemoteViewModel {
    private struct PendingRequest {
        let isModeSwitch: Bool
        let timeoutTask: Task<Void, Never>
    }

    private let webSocket: any WebSocketClientProtocol
    private let pairingService: any PairingServiceProtocol
    private let keychain: any KeychainStoreProtocol
    private let settingsStore: any AppSettingsStoreProtocol
    private let reconnectPolicy: ReconnectPolicy

    private var token: String?
    private var connectionTask: Task<Void, Never>?
    private var connectionGeneration: UInt64 = 0
    private var pendingRequests: [UUID: PendingRequest] = [:]
    private var shouldReconnect = false
    private var connectedAt: Date?
    private var serverRetryDelay: TimeInterval?

    private(set) var connectionState: ConnectionState = .disconnected
    private(set) var displayMode: DisplayMode?
    private(set) var isTransitioning = false
    private(set) var hasToken = false
    private(set) var needsRepairing = false
    private(set) var isPairing = false
    var alertMessage: String?
    var isSettingsPresented = false
    var host: String
    var portText: String
    var pairingCode = ""
    var textInput = ""
    var isTextInputPresented = false

    init(
        webSocket: any WebSocketClientProtocol = WebSocketClient(),
        pairingService: any PairingServiceProtocol = PairingService(),
        keychain: any KeychainStoreProtocol = KeychainStore(),
        settingsStore: any AppSettingsStoreProtocol = AppSettingsStore(),
        reconnectPolicy: ReconnectPolicy = ReconnectPolicy()
    ) {
        self.webSocket = webSocket
        self.pairingService = pairingService
        self.keychain = keychain
        self.settingsStore = settingsStore
        self.reconnectPolicy = reconnectPolicy
        host = settingsStore.loadHost()
        portText = String(settingsStore.loadPort())
        do {
            token = try keychain.readToken()
            hasToken = token != nil
        } catch {
            alertMessage = error.localizedDescription
        }
    }

    func start() {
        guard connectionTask == nil else { return }
        guard hasToken else {
            connectionState = .disconnected
            return
        }
        shouldReconnect = true
        startConnectionTask(disconnectFirst: false)
    }

    func stop() {
        shouldReconnect = false
        connectionGeneration &+= 1
        connectionTask?.cancel()
        connectionTask = nil
        pendingRequests.values.forEach { $0.timeoutTask.cancel() }
        pendingRequests.removeAll()
        connectionState = .disconnected
        Task { await webSocket.disconnect() }
    }

    func appDidBecomeActive() {
        if connectionState != .connected {
            retry()
        }
    }

    func retry() {
        guard hasToken else {
            isSettingsPresented = true
            return
        }
        restartConnection()
    }

    func saveSettingsAndReconnect() {
        do {
            let host = try SettingsValidator.validatedHost(host)
            let port = try SettingsValidator.validatedPort(portText)
            self.host = host
            portText = String(port)
            settingsStore.save(host: host, port: port)
            alertMessage = nil
            if hasToken { restartConnection() }
        } catch {
            alertMessage = error.localizedDescription
        }
    }

    func pair() {
        guard !isPairing else { return }
        Task {
            do {
                let host = try SettingsValidator.validatedHost(host)
                let port = try SettingsValidator.validatedPort(portText)
                let code = try SettingsValidator.validatedPairingCode(pairingCode)
                settingsStore.save(host: host, port: port)
                self.host = host
                portText = String(port)
                isPairing = true
                defer { isPairing = false }

                let configuration = ConnectionConfiguration(host: host, port: port, token: nil)
                let response = try await pairingService.pair(
                    configuration: configuration,
                    code: code,
                    deviceName: UIDevice.current.name
                )
                try keychain.saveToken(response.token)
                token = response.token
                hasToken = true
                needsRepairing = false
                pairingCode = ""
                alertMessage = nil
                restartConnection()
            } catch {
                alertMessage = pairingMessage(for: error)
            }
        }
    }

    func forgetPairing() {
        do {
            try keychain.deleteToken()
            token = nil
            hasToken = false
            needsRepairing = false
            stop()
        } catch {
            alertMessage = error.localizedDescription
        }
    }

    func sendNavigation(_ action: RemoteAction) {
        guard action != .switchMode else { return }
        send(command: RemoteCommand(action: action), timeout: 5, isModeSwitch: false)
    }

    func switchMode(to mode: DisplayMode) {
        guard !pendingRequests.values.contains(where: \.isModeSwitch) else { return }
        let command = RemoteCommand(action: .switchMode, params: CommandParams(mode: mode))
        send(command: command, timeout: 12, isModeSwitch: true)
    }

    func sendTextInput() {
        guard validateTextInput() else { return }
        send(
            command: RemoteCommand(action: .inputText, params: CommandParams(text: textInput)),
            timeout: 5,
            isModeSwitch: false
        )
        textInput = ""
    }

    func deleteBackward() {
        send(command: RemoteCommand(action: .deleteBackward), timeout: 5, isModeSwitch: false)
    }

    func submitTextInput() {
        send(command: RemoteCommand(action: .submitText), timeout: 5, isModeSwitch: false)
    }

    private func validateTextInput() -> Bool {
        guard !textInput.isEmpty else {
            alertMessage = String(localized: "入力する文字を入力してください。")
            return false
        }
        guard textInput.unicodeScalars.count <= 200, !textInput.contains(where: \.isNewline) else {
            alertMessage = String(localized: "文字入力は改行を含めず200文字以内にしてください。")
            return false
        }
        return true
    }

    private func send(command: RemoteCommand, timeout: TimeInterval, isModeSwitch: Bool) {
        guard connectionState.isConnected else { return }
        let timeoutTask = Task { [weak self] in
            do {
                try await Task.sleep(for: .seconds(timeout))
                guard !Task.isCancelled else { return }
                self?.requestTimedOut(command.requestId)
            } catch { }
        }
        pendingRequests[command.requestId] = PendingRequest(
            isModeSwitch: isModeSwitch,
            timeoutTask: timeoutTask
        )
        Task {
            do {
                try await webSocket.send(command)
            } catch {
                completeRequest(command.requestId)
                alertMessage = String(localized: "操作を送信できませんでした。")
                restartConnection()
            }
        }
    }

    private func runConnectionLoop(generation: UInt64) async {
        var attempt = 0

        while isCurrentConnection(generation) && !Task.isCancelled {
            guard let token else {
                connectionState = .disconnected
                return
            }
            do {
                let validatedHost = try SettingsValidator.validatedHost(host)
                let port = try SettingsValidator.validatedPort(portText)
                connectionState = attempt == 0 ? .connecting : .reconnecting(attempt: attempt)
                let configuration = ConnectionConfiguration(host: validatedHost, port: port, token: token)
                try await webSocket.connect(configuration: configuration)
                guard isCurrentConnection(generation), !Task.isCancelled else { return }
                let messages = await webSocket.messages()
                for try await message in messages {
                    guard isCurrentConnection(generation), !Task.isCancelled else { return }
                    try await handle(message)
                }
                if !Task.isCancelled { throw RemoteClientError.disconnected }
            } catch {
                guard isCurrentConnection(generation), !Task.isCancelled else { return }
                await webSocket.disconnect()
                guard isCurrentConnection(generation), !Task.isCancelled else { return }
                if isAuthenticationError(error) {
                    needsRepairing = true
                    connectionState = .failed(message: String(localized: "再ペアリングが必要です。"))
                    isSettingsPresented = true
                    return
                }
                if error as? RemoteClientError == .protocolMismatch {
                    connectionState = .failed(message: RemoteClientError.protocolMismatch.localizedDescription)
                    return
                }
                if let connectedAt, Date().timeIntervalSince(connectedAt) >= 30 {
                    attempt = 0
                }
                self.connectedAt = nil
                attempt += 1
                connectionState = .reconnecting(attempt: attempt)
                let jitter = Double.random(in: 0...0.5)
                let delay = serverRetryDelay ?? reconnectPolicy.delay(forAttempt: attempt, jitter: jitter)
                serverRetryDelay = nil
                do {
                    try await Task.sleep(for: .seconds(delay))
                } catch { return }
            }
        }
    }

    private func handle(_ message: ServerMessage) async throws {
        switch message {
        case let .hello(protocolVersion, _, _, role):
            guard protocolVersion == 1, role == "remote" else {
                throw RemoteClientError.protocolMismatch
            }
            connectedAt = Date()
            connectionState = .connected
            needsRepairing = false
        case let .systemState(mode, transitioning, _, _):
            displayMode = mode
            isTransitioning = transitioning
        case let .commandResult(requestId, ok, error):
            completeRequest(requestId)
            if !ok {
                alertMessage = error?.message ?? String(localized: "操作を完了できませんでした。")
            }
        case let .error(error):
            if error.code == "unauthorized" {
                throw RemoteClientError.unauthorized
            }
            if error.code == "unsupported_protocol" {
                throw RemoteClientError.protocolMismatch
            }
            alertMessage = error.message
        case let .shutdown(retryAfterSeconds):
            serverRetryDelay = TimeInterval(max(0, retryAfterSeconds))
            await webSocket.disconnect()
        case .unknown:
            break
        }
    }

    private func restartConnection() {
        shouldReconnect = true
        pendingRequests.values.forEach { $0.timeoutTask.cancel() }
        pendingRequests.removeAll()
        startConnectionTask(disconnectFirst: true)
    }

    private func startConnectionTask(disconnectFirst: Bool) {
        connectionGeneration &+= 1
        let generation = connectionGeneration
        connectionTask?.cancel()
        connectionTask = Task { [weak self] in
            guard let self else { return }
            if disconnectFirst {
                await webSocket.disconnect()
            }
            guard isCurrentConnection(generation), !Task.isCancelled else { return }
            await runConnectionLoop(generation: generation)
            if connectionGeneration == generation {
                connectionTask = nil
            }
        }
    }

    private func isCurrentConnection(_ generation: UInt64) -> Bool {
        shouldReconnect && connectionGeneration == generation
    }

    private func requestTimedOut(_ requestId: UUID) {
        guard pendingRequests.removeValue(forKey: requestId) != nil else { return }
        alertMessage = String(localized: "操作を完了できませんでした。")
    }

    private func completeRequest(_ requestId: UUID) {
        pendingRequests.removeValue(forKey: requestId)?.timeoutTask.cancel()
    }

    private func isAuthenticationError(_ error: Error) -> Bool {
        guard let remoteError = error as? RemoteClientError else { return false }
        return remoteError == .unauthorized || remoteError == .missingToken
    }

    private func pairingMessage(for error: Error) -> String {
        if error as? RemoteClientError == .unauthorized {
            return String(localized: "ペアリングコードを確認してください。")
        }
        return error.localizedDescription
    }
}
