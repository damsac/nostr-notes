import SwiftUI

@main
struct RMPApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "hammer.fill")
                .font(.system(size: 48))
                .foregroundStyle(.blue)
            Text("RMP App")
                .font(.title)
            Text("Rust core + SwiftUI shell")
                .foregroundStyle(.secondary)
        }
        .padding()
    }
}
