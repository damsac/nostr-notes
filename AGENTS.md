# nostr-notes

Nostr note reader. RMP (Rust Multi-Platform) app: Rust core (nostr-sdk + SQLite) -> UniFFI FFI -> SwiftUI / Compose native shells.

Relay: `wss://nostr.damsac.studio`

## First thing: run agent-brief
```bash
./scripts/agent-brief
```

## Commands (justfile is the surface)
```
just test              # Run all Rust tests
just build             # Build all workspace crates
just clippy            # Lint (deny warnings)
just fmt               # Check formatting
just ios-build         # Build iOS (xcframework + UniFFI Swift bindings)
just android-build     # Build Android (cargo-ndk + UniFFI Kotlin bindings)
just pre-merge         # Full CI check locally (fmt + clippy + test)
just pre-commit        # Quick check (fmt + clippy)
```

## Architecture

- `crates/core/`: ALL business logic. Pure Rust. nostr-sdk for relay communication, rusqlite for persistence. No platform deps.
- `crates/ffi/`: UniFFI bindings. Thin `Ffi*` wrappers around core types. No logic.
- `crates/uniffi-bindgen/`: Binding generator binary.
- `ios/`: SwiftUI app. Thin UI shell. Renders state from Rust.
- `android/`: Kotlin/Compose app. Thin UI shell. Renders state from Rust.
- State flows unidirectionally: Native -> Action -> Rust -> State -> Native renders.

## Key files

- `justfile`: command surface (start here)
- `scripts/`: build tooling (don't modify without reading docs/)
- `invariants/invariants.toml`: architecture rules (machine-checkable)
- `docs/`: read_when frontmatter tells you which docs matter for your task

## Invariants

See `invariants/invariants.toml`. The critical rules:

1. All business logic in `crates/core/`. No platform-specific code in core.
2. Native layers are thin UI shells. No business logic in Swift/Kotlin.
3. Unidirectional state flow. Rust core is the single source of truth.
4. UniFFI bindings are generated, not hand-written. FFI crate uses `Ffi*` wrappers.
5. Tests live alongside the code they test.
