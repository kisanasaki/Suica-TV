import Foundation

struct ReconnectPolicy: Sendable {
    private let delays: [TimeInterval] = [1, 2, 4, 8, 10]

    func delay(forAttempt attempt: Int, jitter: TimeInterval = 0) -> TimeInterval {
        let index = max(0, min(attempt - 1, delays.count - 1))
        return delays[index] + min(max(jitter, 0), 0.5)
    }
}
