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
            .navigationTitle("Notes")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    if !notes.isEmpty {
                        Text("\(notes.count)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
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
            if core == nil {
                errorMessage = "Failed to connect. Check your network and try again."
            } else {
                errorMessage = "Could not load notes. Pull to refresh."
            }
        }
        isLoading = false
    }
}

struct NoteRow: View {
    let note: FfiNote

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 6) {
                Text(note.displayName)
                    .font(.subheadline)
                    .fontWeight(.medium)
                Spacer()
                Text(note.relativeTime)
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }
            Text(note.content)
                .font(.body)
                .lineLimit(6)
                .foregroundStyle(.primary)
        }
        .padding(.vertical, 4)
    }
}
