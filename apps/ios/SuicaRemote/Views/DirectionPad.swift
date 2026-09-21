import SwiftUI
import UIKit

@MainActor
struct DirectionPad: View {
    let isEnabled: Bool
    let action: (RemoteAction) -> Void

    var body: some View {
        Grid(horizontalSpacing: 12, verticalSpacing: 12) {
            GridRow {
                Color.clear.frame(width: 72, height: 64)
                directionButton(.up, systemName: "chevron.up", label: String(localized: "上"))
                Color.clear.frame(width: 72, height: 64)
            }
            GridRow {
                directionButton(.left, systemName: "chevron.left", label: String(localized: "左"))
                actionButton(.select, title: "OK", label: String(localized: "決定"))
                directionButton(.right, systemName: "chevron.right", label: String(localized: "右"))
            }
            GridRow {
                Color.clear.frame(width: 72, height: 64)
                directionButton(.down, systemName: "chevron.down", label: String(localized: "下"))
                Color.clear.frame(width: 72, height: 64)
            }
        }
        .disabled(!isEnabled)
        .opacity(isEnabled ? 1 : 0.45)
    }

    private func directionButton(_ remoteAction: RemoteAction, systemName: String, label: String) -> some View {
        Button {
            feedback()
            action(remoteAction)
        } label: {
            Image(systemName: systemName)
                .font(.title2.bold())
                .frame(width: 72, height: 64)
        }
        .buttonStyle(.bordered)
        .accessibilityLabel(label)
        .accessibilityHint(String(localized: "テレビ画面の選択位置を移動します"))
        .accessibilityIdentifier("remote-\(remoteAction.rawValue)")
    }

    private func actionButton(
        _ remoteAction: RemoteAction,
        title: String,
        label: String
    ) -> some View {
        Button {
            feedback()
            action(remoteAction)
        } label: {
            Text(title)
                .font(.headline)
                .frame(width: 72, height: 64)
        }
        .buttonStyle(.borderedProminent)
        .accessibilityLabel(label)
        .accessibilityIdentifier("remote-\(remoteAction.rawValue)")
    }

    private func feedback() {
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }
}
