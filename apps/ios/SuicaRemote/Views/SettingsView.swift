//
// Core接続先とペアリングコードを編集・検証する設定画面。
// 保存とペアリングの実処理はViewModelへ委譲し、秘密情報を永続状態として保持しない。
//

import SwiftUI

@MainActor
struct SettingsView: View {
    @Bindable var viewModel: RemoteViewModel
    @Environment(\.dismiss) private var dismiss

    private var showsPairing: Bool {
        !viewModel.hasToken || viewModel.needsRepairing
    }

    var body: some View {
        NavigationStack {
            Form {
                Section(String(localized: "接続先")) {
                    TextField(String(localized: "ホスト名またはIPv4"), text: $viewModel.host)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                        .accessibilityIdentifier("host-field")
                    TextField(String(localized: "ポート"), text: $viewModel.portText)
                        .keyboardType(.numberPad)
                        .accessibilityIdentifier("port-field")
                    Button(String(localized: "保存して再接続")) {
                        viewModel.saveSettingsAndReconnect()
                    }
                    .accessibilityIdentifier("reconnect-button")
                }

                Section(String(localized: "状態")) {
                    LabeledContent(String(localized: "接続"), value: viewModel.connectionState.label)
                    LabeledContent(
                        String(localized: "現在モード"),
                        value: viewModel.displayMode?.localizedName ?? String(localized: "不明")
                    )
                    if viewModel.isTransitioning {
                        Label(String(localized: "モードを切り替えています"), systemImage: "arrow.triangle.2.circlepath")
                    }
                    HStack {
                        modeButton(.tv)
                        modeButton(.pc)
                    }
                }

                if showsPairing {
                    Section {
                        SecureField(String(localized: "6桁コード"), text: $viewModel.pairingCode)
                            .keyboardType(.numberPad)
                            .accessibilityIdentifier("pairing-code-field")
                        Button {
                            viewModel.pair()
                        } label: {
                            if viewModel.isPairing {
                                ProgressView()
                            } else {
                                Text(String(localized: "ペアリングする"))
                            }
                        }
                        .disabled(viewModel.isPairing)
                        .accessibilityIdentifier("pair-button")
                    } header: {
                        Text(String(localized: "ペアリング"))
                    } footer: {
                        Text(String(localized: "Raspberry Piに表示された6桁コードを入力してください。"))
                    }
                } else {
                    Section(String(localized: "ペアリング")) {
                        Label(String(localized: "登録済み"), systemImage: "checkmark.shield.fill")
                            .foregroundStyle(.green)
                        Button(String(localized: "ペアリングを解除"), role: .destructive) {
                            viewModel.forgetPairing()
                        }
                    }
                }
            }
            .navigationTitle(String(localized: "設定"))
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button(String(localized: "完了")) { dismiss() }
                }
            }
            .alert(
                String(localized: "Suica Remote"),
                isPresented: Binding(
                    get: { viewModel.alertMessage != nil },
                    set: { if !$0 { viewModel.alertMessage = nil } }
                )
            ) {
                Button(String(localized: "OK"), role: .cancel) { viewModel.alertMessage = nil }
            } message: {
                Text(viewModel.alertMessage ?? "")
            }
        }
    }

    private func modeButton(_ mode: DisplayMode) -> some View {
        Button(mode.localizedName) { viewModel.switchMode(to: mode) }
            .buttonStyle(.bordered)
            .frame(maxWidth: .infinity)
            .disabled(
                !viewModel.connectionState.isConnected ||
                viewModel.isTransitioning ||
                viewModel.displayMode == mode
            )
            .accessibilityIdentifier("mode-\(mode.rawValue)")
    }
}
