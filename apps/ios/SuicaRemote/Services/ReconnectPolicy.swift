//
// WebSocket再接続の指数的backoffと上限を計算する。
// 待機方法や接続状態は持たず、試行回数からdelayだけを返す。
//

import Foundation

/// 連続失敗時の待機時間を上限付き指数backoffとして提供する。
struct ReconnectPolicy: Sendable {
    private let delays: [TimeInterval]

    init(delays: [TimeInterval] = [1, 2, 4, 8, 10]) {
        precondition(!delays.isEmpty)
        self.delays = delays
    }

    func delay(forAttempt attempt: Int, jitter: TimeInterval = 0) -> TimeInterval {
        let index = max(0, min(attempt - 1, delays.count - 1))
        return delays[index] + min(max(jitter, 0), 0.5)
    }
}
