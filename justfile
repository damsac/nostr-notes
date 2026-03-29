# RMP project justfile
# Change this to your FFI crate name:
ffi_crate := "nostr-notes-ffi"

# === Core ===

test:
    cargo test --workspace

build:
    cargo build --workspace

check:
    cargo check --workspace

clippy:
    cargo clippy --workspace -- -D warnings

fmt:
    cargo fmt --all -- --check

fmt-fix:
    cargo fmt --all

# === iOS ===

ios-build:
    ./scripts/ios-build --crate-name {{ffi_crate}}

ios-build-release:
    ./scripts/ios-build --crate-name {{ffi_crate}} --release

# === Android ===

android-build:
    ./scripts/android-build --crate-name {{ffi_crate}}

android-build-release:
    ./scripts/android-build --crate-name {{ffi_crate}} --release

android-apk: android-build
    cd android && gradle assembleDebug

android-install: android-apk
    cd android && gradle installDebug

# === CI / QA ===

pre-commit: fmt clippy

pre-merge: fmt clippy test
