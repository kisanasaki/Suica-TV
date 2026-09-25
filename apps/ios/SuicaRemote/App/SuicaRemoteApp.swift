//
// Suica Remoteアプリケーションのエントリーポイント。
// 共有ViewModelを生成し、最上位のRemoteViewへ注入する。
//

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
