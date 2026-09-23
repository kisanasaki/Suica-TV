import XCTest
@testable import SuicaRemote

final class ModelContractTests: XCTestCase {
    func testNavigationCommandMatchesContract() throws {
        let id = UUID(uuidString: "550e8400-e29b-41d4-a716-446655440000")!
        let data = try JSONEncoder().encode(RemoteCommand(requestId: id, action: .up))
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(object["type"] as? String, "remote.command")
        XCTAssertEqual(object["requestId"] as? String, id.uuidString.uppercased())
        XCTAssertEqual(object["action"] as? String, "navigation.up")
        XCTAssertNil(object["params"])
    }

    func testSwitchModeCommandIncludesParams() throws {
        let command = RemoteCommand(action: .switchMode, params: CommandParams(mode: .pc))
        let data = try JSONEncoder().encode(command)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        let params = try XCTUnwrap(object["params"] as? [String: String])
        XCTAssertEqual(params["mode"], "pc")
    }

    func testUnicodeTextCommandIncludesOnlyTextParam() throws {
        let command = RemoteCommand(action: .inputText, params: CommandParams(text: "すいか🍉"))
        let data = try JSONEncoder().encode(command)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(object["action"] as? String, "input.text")
        let params = try XCTUnwrap(object["params"] as? [String: String])
        XCTAssertEqual(params, ["text": "すいか🍉"])
    }

    func testScrollCommandIncludesDeltas() throws {
        let command = RemoteCommand(action: .scroll, params: CommandParams(dx: -120, dy: 360))
        let data = try JSONEncoder().encode(command)
        let object = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
        XCTAssertEqual(object["action"] as? String, "pointer.scroll")
        let params = try XCTUnwrap(object["params"] as? [String: Int])
        XCTAssertEqual(params, ["dx": -120, "dy": 360])
    }

    func testPointerCommandsMatchContract() throws {
        let movement = RemoteCommand(action: .pointerMove, params: CommandParams(dx: 12, dy: -8))
        let movementObject = try XCTUnwrap(
            JSONSerialization.jsonObject(with: JSONEncoder().encode(movement)) as? [String: Any]
        )
        XCTAssertEqual(movementObject["action"] as? String, "pointer.move")
        XCTAssertEqual(movementObject["params"] as? [String: Int], ["dx": 12, "dy": -8])

        let clickObject = try XCTUnwrap(
            JSONSerialization.jsonObject(
                with: JSONEncoder().encode(RemoteCommand(action: .pointerClick))
            ) as? [String: Any]
        )
        XCTAssertEqual(clickObject["action"] as? String, "pointer.click")
        XCTAssertNil(clickObject["params"])
    }

    func testDecodesAllServerMessagesAndIgnoresAddedFields() throws {
        let decoder = JSONDecoder()
        let hello = try decoder.decode(ServerMessage.self, from: Data(#"{"type":"server.hello","protocolVersion":1,"serverVersion":"0.1.0","connectionId":"550e8400-e29b-41d4-a716-446655440000","role":"remote","future":true}"#.utf8))
        guard case let .hello(version, _, _, role) = hello else { return XCTFail("hello expected") }
        XCTAssertEqual(version, 1)
        XCTAssertEqual(role, "remote")

        let state = try decoder.decode(ServerMessage.self, from: Data(#"{"type":"system.state","mode":"tv","transitioning":false,"changedAt":"2026-09-21T00:00:00.123Z"}"#.utf8))
        guard case let .systemState(mode, transitioning, _, _) = state else { return XCTFail("state expected") }
        XCTAssertEqual(mode, .tv)
        XCTAssertFalse(transitioning)

        let result = try decoder.decode(ServerMessage.self, from: Data(#"{"type":"command.result","requestId":"550e8400-e29b-41d4-a716-446655440000","ok":true}"#.utf8))
        guard case let .commandResult(_, ok, _) = result else { return XCTFail("result expected") }
        XCTAssertTrue(ok)

        let unknown = try decoder.decode(ServerMessage.self, from: Data(#"{"type":"future.message","value":1}"#.utf8))
        XCTAssertEqual(unknown, .unknown(type: "future.message"))
    }
}
