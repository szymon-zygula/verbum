AGENTS.md for verbum (Rust 2024)
Use latest stable Rust; rustup toolchain stable recommended
Build/Lint/Test
Build: cargo build; Release: cargo build --release
Tests: cargo test (CI uses cargo build && cargo test)
Single test: cargo test <pattern> -- --nocapture
Integration test: cargo test --test <file_stem> <pattern>
Format: cargo fmt --all; check: cargo fmt --all -- --check
Lint: cargo clippy --all-targets --all-features -D warnings
Code Style
Imports: group std, external, crate; no glob imports; prefer explicit use; keep crate::… paths
Formatting: run cargo fmt before commits; keep diffs minimal
Types: explicit at public boundaries; prefer borrowing; avoid unnecessary clones
Naming: modules/funcs/vars snake_case; types/traits CamelCase; constants SCREAMING_SNAKE_CASE; type params T/U/E
Errors: return anyhow::Result<T> in app code; bubble with ?; add context with anyhow::Context; avoid unwrap/panic (except tests/prototypes)
Result/Option: map/and_then when clearer; avoid silent .ok()
Safety: no unsafe; use iterators/itertools for clarity
Concurrency: use rayon par_iter where helpful; avoid shared mutable state; require Send+Sync where appropriate
Docs/Tests: document public items with ///; tests in #[cfg(test)]; keep unit tests small
Cursor/Copilot/CI: no Cursor or Copilot rules; CI uses cargo build/test in .github/workflows/rust.yml