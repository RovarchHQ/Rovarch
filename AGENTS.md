# Working on Rovarch

Read [AI_USAGE_POLICY.md](AI_USAGE_POLICY.md) before making changes.

Rovarch is a Linux launcher for Roblox Player and Studio. The UI is
Rust on GTK4 with libadwaita; it drives [Rovarch CORE] to install and
run the game.

[Rovarch CORE]: https://github.com/RovarchHQ/rovarch-core

## Principles

- Code is self-explanatory. No comments that restate the code.
- Prefer the existing abstractions over a new parallel path.
- A change is done when it compiles and passes `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test`.

## Layout

- UI components live in `src/components/`, exported from `src/components.rs`.
- A page owns a folder `src/pages/<name>/`. The page goes in `mod.rs`;
  helper functions only that page uses go in their own files inside the
  folder. Pages are exported from `src/pages.rs`.
- Shared, page-independent helpers go in `src/utils.rs`.
- Backend work (installing, downloading, running the game) goes in
  `src/backend/`, with its public actions in `src/backend.rs`. The backend
  reaches the UI only through `AppState`.
- User-visible state goes in `src/state.rs` behind its abstractions. Add a
  property there when the UI needs new state; otherwise keep using the
  existing abstractions.
- UI strings go through the i18n layer. English is the built-in default;
  Arabic lives in `src/locales/ar.toml`. When you add or change a user-facing
  string, keep its key in `ar.toml` too. A missing key leaves just that line
  in English, but a malformed file (such as a duplicated key) fails the whole
  parse, and every translation — and RTL — silently drops to English.

## Background work

Long-running work runs off the main thread in `src/backend/background.rs`:

- `run_task` is for work the user started. It marks Rovarch busy, reports
  progress, and opens an error dialog on failure.
- `run_quietly` is for work the user should not notice. It blocks nothing,
  and the caller handles the `Result`, usually by logging it.
