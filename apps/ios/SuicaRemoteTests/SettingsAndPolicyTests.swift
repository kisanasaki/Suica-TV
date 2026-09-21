import XCTest
@testable import SuicaRemote

final class SettingsAndPolicyTests: XCTestCase {
    func testConnectionURLs() throws {
        let configuration = ConnectionConfiguration(host: "raspberrypi.local", port: 3030, token: "secret")
        XCTAssertEqual(
            configuration.webSocketURL?.absoluteString,
            "ws://raspberrypi.local:3030/ws?role=remote&protocolVersion=1"
        )
        XCTAssertEqual(configuration.pairingURL?.absoluteString, "http://raspberrypi.local:3030/api/v1/pair")
    }

    func testSettingsValidation() throws {
        XCTAssertEqual(try SettingsValidator.validatedHost(" raspberrypi.local "), "raspberrypi.local")
        XCTAssertEqual(try SettingsValidator.validatedHost("192.168.1.10"), "192.168.1.10")
        XCTAssertThrowsError(try SettingsValidator.validatedHost("http://raspberrypi.local/path"))
        XCTAssertEqual(try SettingsValidator.validatedPort("3030"), 3030)
        XCTAssertThrowsError(try SettingsValidator.validatedPort("65536"))
        XCTAssertEqual(try SettingsValidator.validatedPairingCode("123456"), "123456")
        XCTAssertThrowsError(try SettingsValidator.validatedPairingCode("12345a"))
    }

    func testReconnectPolicyBackoffAndJitterLimit() {
        let policy = ReconnectPolicy()
        XCTAssertEqual((1...6).map { policy.delay(forAttempt: $0) }, [1, 2, 4, 8, 10, 10])
        XCTAssertEqual(policy.delay(forAttempt: 1, jitter: 1), 1.5)
        XCTAssertEqual(policy.delay(forAttempt: 2, jitter: -1), 2)
    }

    func testSettingsStoreDoesNotPersistToken() {
        let suite = "SettingsAndPolicyTests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suite)!
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = AppSettingsStore(defaults: defaults)
        store.save(host: "pi.local", port: 4040)
        XCTAssertEqual(store.loadHost(), "pi.local")
        XCTAssertEqual(store.loadPort(), 4040)
        XCTAssertNil(defaults.string(forKey: "token"))
        let persistedKeys = Set(defaults.persistentDomain(forName: suite)?.keys.map { $0 } ?? [])
        XCTAssertEqual(persistedKeys, ["suicaRemote.host", "suicaRemote.port"])
    }
}
