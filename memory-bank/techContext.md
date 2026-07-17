# Tech Context: Halvora

## Technologies Used
| Technology | Version | Purpose |
|---|---|---|
| Rust | 2024 edition | Core language |
| Iced | 0.14.0 | GUI framework |
| BitStamp API | — | Exchange data source |
| Mempool.space API | — | Blockchain data source |
| SQLite | (via rusqlite/sqlx) | Local data persistence |

## Development Setup
- **Build**: `cargo build` / `cargo run`
- **Project root**: `/home/dockport/DOCK-HQ/DEV/halvora`
- **Dependencies file**: `Cargo.toml` (currently only has `iced = "0.14.0"`)
- **No tests written yet** — testing framework to be decided

## Iced 0.14.x Key Concepts
- **Sandbox**: Simple boilerplate for apps without async — won't work for Halvora since we need API calls
- **Application**: Full-featured trait with `Executor`, `Command`, and `Subscription` for async operations
- **Widgets**: `Container`, `Column`, `Row`, `Text`, `Button`, `Scrollable`, `Canvas`, `PaneGrid`
- **Theming**: Iced's built-in `Theme` enum or custom `theme::Application` trait implementations
- **Canvas**: Custom 2D rendering via `Canvas` widget + `Program` trait for chart drawing

## Technical Constraints
- **Rust edition 2024**: This is very new (unstable/experimental) — may need to verify Iced compatibility or switch to edition 2021 if issues arise
- **Iced 0.14.0**: API is still evolving; breaking changes between versions are common
- **Desktop only**: No web or mobile target (Iced does have wasm support, but out of scope)
- **Pre-compiled database**: Need a strategy for bundling SQLite database files with the binary (include via `include_bytes!` or similar)

## Dependencies Needed (to be added to Cargo.toml)
Based on the architecture plan, these crates will likely be needed:
- `iced = "0.14.0"` (already added)
- `rusqlite` or `sqlx` — SQLite access
- `reqwest` — HTTP client for BitStamp and Mempool APIs
- `serde` / `serde_json` — JSON parsing for API responses
- `chrono` — Date/time handling for halving events and candle data
- `tokio` — Async runtime (if not already included via Iced)

## Tool Usage Patterns
- Build commands: `cargo build`, `cargo run`, `cargo check`
- Code organization: Modules under `src/Modules/` following the existing structure
- No linter/formatter config yet (rustfmt default)
- Git origin: `https://github.com/DOCKPORT/Halvora.git`