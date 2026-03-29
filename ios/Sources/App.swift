import SwiftUI

@main
struct NostrNotesApp: App {
    var body: some Scene {
        WindowGroup {
            NoteListView()
        }
    }
}

struct NoteListView: View {
    @State private var notes: [FfiNote] = []
    @State private var isLoading = false
    @State private var errorMessage: String?
    @State private var core: AppCore?

    private let relayUrl = "wss://nostr.damsac.studio"

    var body: some View {
        NavigationStack {
            Group {
                if isLoading && notes.isEmpty {
                    ProgressView("Connecting to relays...")
                } else if let error = errorMessage, notes.isEmpty {
                    VStack(spacing: 12) {
                        Image(systemName: "wifi.exclamationmark")
                            .font(.system(size: 40))
                            .foregroundStyle(.secondary)
                        Text(error)
                            .foregroundStyle(.secondary)
                            .multilineTextAlignment(.center)
                        Button("Retry") { Task { await loadNotes() } }
                    }
                    .padding()
                } else {
                    List(notes, id: \.id) { note in
                        NoteRow(note: note)
                    }
                    .refreshable { await loadNotes() }
                }
            }
            .navigationTitle("Nostr Notes")
            .task { await loadNotes() }
        }
    }

    private func getOrCreateCore() throws -> AppCore {
        if let existing = core {
            return existing
        }
        let dataDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0].path()
        let newCore = try AppCore(relayUrl: relayUrl, dataDir: dataDir)
        core = newCore
        return newCore
    }

    func loadNotes() async {
        isLoading = true
        errorMessage = nil
        do {
            let appCore = try getOrCreateCore()
            let fetched = try appCore.fetchGlobalNotes(limit: 50)
            notes = fetched
        } catch {
            // If core creation failed, clear it so next attempt retries
            if core == nil {
                errorMessage = "Failed to connect: \(error.localizedDescription)"
            } else {
                errorMessage = error.localizedDescription
            }
        }
        isLoading = false
    }
}

struct NoteRow: View {
    let note: FfiNote

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(note.content)
                .lineLimit(5)
            HStack {
                Text(String(note.pubkey.prefix(12)) + "...")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
                Text(formatTimestamp(note.createdAt))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }

    func formatTimestamp(_ ts: Int64) -> String {
        let date = Date(timeIntervalSince1970: TimeInterval(ts))
        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: .now)
    }
}
