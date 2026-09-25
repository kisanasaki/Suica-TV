//
// 接続状態を色、アイコン、ラベルで表示する小さなSwiftUI部品。
// 状態判定や再接続処理は行わず、ConnectionStateの表示属性だけを利用する。
//

import SwiftUI

@MainActor
struct ConnectionStatusView: View {
    let state: ConnectionState

    private var color: Color {
        switch state {
        case .connected: .green
        case .connecting, .reconnecting: .orange
        case .disconnected: .secondary
        case .failed: .red
        }
    }

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: state.symbolName)
                .symbolEffect(.pulse, isActive: state == .connecting)
            Text(state.label)
                .font(.headline)
        }
        .foregroundStyle(color)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(color.opacity(0.12), in: Capsule())
        .accessibilityElement(children: .combine)
        .accessibilityLabel(String(localized: "接続状態：\(state.label)"))
        .accessibilityIdentifier("connection-status")
    }
}
