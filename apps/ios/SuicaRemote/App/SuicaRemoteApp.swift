import SwiftUI

@main
@MainActor
struct SuicaRemoteApp: App {
    @State private var viewModel = RemoteViewModel()

    var body: some Scene {
        WindowGroup {
            RemoteView(viewModel: viewModel)
        }
    }
}
