import SwiftUI

@MainActor
struct RemoteView: View {
    @Bindable var viewModel: RemoteViewModel
    @Environment(\.scenePhase) private var scenePhase

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 28) {
                    ConnectionStatusView(state: viewModel.connectionState)

                    if case let .failed(message) = viewModel.connectionState {
                        Text(message)
                            .font(.subheadline)
                            .foregroundStyle(.red)
                            .multilineTextAlignment(.center)
                    }

                    DirectionPad(isEnabled: viewModel.connectionState.isConnected) {
                        viewModel.sendNavigation($0)
                    }

                    HStack(spacing: 16) {
                        utilityButton(
                            title: String(localized: "戻る"),
                            systemName: "arrow.uturn.backward",
                            action: .back
                        )
                        utilityButton(
                            title: String(localized: "ホーム"),
                            systemName: "house.fill",
                            action: .home
                        )
                    }
                    .disabled(!viewModel.connectionState.isConnected)
                    .opacity(viewModel.connectionState.isConnected ? 1 : 0.45)

                    if !viewModel.connectionState.isConnected, viewModel.hasToken {
                        Button(String(localized: "今すぐ再接続")) { viewModel.retry() }
                            .buttonStyle(.bordered)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding()
            }
            .navigationTitle(String(localized: "Suica Remote"))
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        viewModel.isSettingsPresented = true
                    } label: {
                        Label(String(localized: "設定"), systemImage: "gearshape")
                    }
                    .accessibilityIdentifier("settings-button")
                }
            }
            .sheet(isPresented: $viewModel.isSettingsPresented) {
                SettingsView(viewModel: viewModel)
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
        .task { viewModel.start() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active { viewModel.appDidBecomeActive() }
        }
    }

    private func utilityButton(title: String, systemName: String, action: RemoteAction) -> some View {
        Button {
            UIImpactFeedbackGenerator(style: .light).impactOccurred()
            viewModel.sendNavigation(action)
        } label: {
            Label(title, systemImage: systemName)
                .frame(maxWidth: .infinity, minHeight: 52)
        }
        .buttonStyle(.bordered)
        .accessibilityHint(String(localized: "テレビ画面へ操作を送信します"))
        .accessibilityIdentifier("remote-\(action.rawValue)")
    }
}
