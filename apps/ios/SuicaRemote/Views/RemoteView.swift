import SwiftUI

@MainActor
struct RemoteView: View {
    @Bindable var viewModel: RemoteViewModel
    @Environment(\.scenePhase) private var scenePhase
    @State private var pointerSensitivity = 1.0

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

                    ScrollPad(isEnabled: viewModel.canUsePointer) { dx, dy in
                        viewModel.queueScroll(dx: dx, dy: dy)
                    } onEnded: {
                        viewModel.flushScroll()
                    }

                    PointerPad(
                        isEnabled: viewModel.canUsePointer,
                        sensitivity: $pointerSensitivity
                    ) { dx, dy in
                        viewModel.queuePointerMove(dx: dx, dy: dy)
                    } onEnded: {
                        viewModel.flushPointerMove()
                    } onClick: {
                        viewModel.clickPointer()
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

                    Button {
                        UIImpactFeedbackGenerator(style: .light).impactOccurred()
                        viewModel.isTextInputPresented = true
                    } label: {
                        Label(String(localized: "文字入力"), systemImage: "keyboard")
                            .frame(maxWidth: .infinity, minHeight: 52)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.connectionState.isConnected)
                    .opacity(viewModel.connectionState.isConnected ? 1 : 0.45)
                    .accessibilityHint(String(localized: "YouTubeの検索欄へ文字を送信します"))
                    .accessibilityIdentifier("text-input-button")

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
            .sheet(isPresented: $viewModel.isTextInputPresented) {
                TextInputView(viewModel: viewModel)
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

private struct ScrollPad: View {
    let isEnabled: Bool
    let onScroll: (Int, Int) -> Void
    let onEnded: () -> Void
    @State private var previousTranslation: CGSize = .zero

    var body: some View {
        RoundedRectangle(cornerRadius: 22)
            .fill(.secondary.opacity(0.12))
            .overlay {
                VStack(spacing: 8) {
                    Image(systemName: "hand.draw")
                        .font(.title)
                    Text(String(localized: "スクロール"))
                        .font(.headline)
                    Text(String(localized: "指で上下に動かします"))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity, minHeight: 130)
            .contentShape(Rectangle())
            .gesture(
                DragGesture(minimumDistance: 1)
                    .onChanged { value in
                        guard isEnabled else { return }
                        let deltaX = value.translation.width - previousTranslation.width
                        let deltaY = value.translation.height - previousTranslation.height
                        previousTranslation = value.translation
                        onScroll(Int((-deltaX * 3).rounded()), Int((-deltaY * 3).rounded()))
                    }
                    .onEnded { _ in
                        previousTranslation = .zero
                        guard isEnabled else { return }
                        onEnded()
                    }
            )
            .opacity(isEnabled ? 1 : 0.45)
            .accessibilityLabel(String(localized: "YouTubeをスクロール"))
            .accessibilityHint(String(localized: "指で上下に動かしてテレビ画面をスクロールします"))
            .accessibilityIdentifier("scroll-pad")
    }
}

private struct PointerPad: View {
    let isEnabled: Bool
    @Binding var sensitivity: Double
    let onMove: (Int, Int) -> Void
    let onEnded: () -> Void
    let onClick: () -> Void
    @State private var previousTranslation: CGSize = .zero

    var body: some View {
        VStack(spacing: 12) {
            RoundedRectangle(cornerRadius: 22)
                .fill(.secondary.opacity(0.12))
                .overlay {
                    VStack(spacing: 8) {
                        Image(systemName: "cursorarrow.motionlines")
                            .font(.title)
                        Text(String(localized: "ポインター"))
                            .font(.headline)
                        Text(String(localized: "指でカーソルを動かします"))
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, minHeight: 150)
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 1)
                        .onChanged { value in
                            guard isEnabled else { return }
                            let deltaX = value.translation.width - previousTranslation.width
                            let deltaY = value.translation.height - previousTranslation.height
                            previousTranslation = value.translation
                            onMove(
                                Int((deltaX * sensitivity).rounded()),
                                Int((deltaY * sensitivity).rounded())
                            )
                        }
                        .onEnded { _ in
                            previousTranslation = .zero
                            guard isEnabled else { return }
                            onEnded()
                        }
                )
                .accessibilityIdentifier("pointer-pad")

            HStack {
                Text(String(localized: "感度"))
                Slider(value: $sensitivity, in: 0.5...2.0, step: 0.25)
                    .accessibilityIdentifier("pointer-sensitivity")
            }

            Button {
                UIImpactFeedbackGenerator(style: .light).impactOccurred()
                onClick()
            } label: {
                Label(String(localized: "クリック"), systemImage: "cursorarrow.click")
                    .frame(maxWidth: .infinity, minHeight: 44)
            }
            .buttonStyle(.bordered)
            .disabled(!isEnabled)
            .accessibilityIdentifier("pointer-click")
        }
        .opacity(isEnabled ? 1 : 0.45)
        .accessibilityLabel(String(localized: "テレビ画面のポインター操作"))
    }
}

@MainActor
private struct TextInputView: View {
    @Bindable var viewModel: RemoteViewModel
    @Environment(\.dismiss) private var dismiss
    @FocusState private var isFocused: Bool
    @State private var isSearchFieldConfirmed = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField(
                        String(localized: "検索文字列"),
                        text: $viewModel.textInput,
                        axis: .vertical
                    )
                    .lineLimit(1...4)
                    .focused($isFocused)
                    .accessibilityIdentifier("text-input-field")
                } footer: {
                    Text(String(localized: "先にテレビ画面の検索欄へフォーカスを移動してください。最大200文字です。"))
                }

                Section {
                    Toggle(
                        String(localized: "テレビ画面の検索欄を選択済み"),
                        isOn: $isSearchFieldConfirmed
                    )
                    .accessibilityIdentifier("search-field-confirmation")
                }

                Section {
                    Button {
                        viewModel.sendTextInput()
                    } label: {
                        Label(String(localized: "文字を送信"), systemImage: "text.cursor")
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .disabled(
                        viewModel.textInput.isEmpty
                            || !isSearchFieldConfirmed
                            || !viewModel.connectionState.isConnected
                    )
                    .accessibilityIdentifier("send-text-button")

                    Button {
                        viewModel.deleteBackward()
                    } label: {
                        Label(String(localized: "1文字削除"), systemImage: "delete.left")
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .disabled(!isSearchFieldConfirmed || !viewModel.connectionState.isConnected)
                    .accessibilityIdentifier("delete-text-button")

                    Button {
                        viewModel.submitTextInput()
                    } label: {
                        Label(String(localized: "検索を実行"), systemImage: "magnifyingglass")
                            .frame(maxWidth: .infinity, minHeight: 44)
                    }
                    .disabled(!isSearchFieldConfirmed || !viewModel.connectionState.isConnected)
                    .accessibilityIdentifier("submit-text-button")
                }
            }
            .navigationTitle(String(localized: "文字入力"))
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button(String(localized: "キャンセル")) { dismiss() }
                }
            }
            .onAppear { isFocused = true }
        }
    }
}
