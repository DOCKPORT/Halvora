# Progress: Halvora

## What Works
- Cargo project scaffolding (`Cargo.toml` with Iced dependency)
- Project directory structure (`src/main.rs`, `src/Modules/API/BitStamp/`, `src/Modules/API/Mempool/`, `src/Modules/UI/mainwindow/`)
- GitHub repository setup (origin: https://github.com/DOCKPORT/Halvora.git)
- README.md with project description
- Memory Bank initialized (all 6 core files)

## What's Left to Build

### UI Layer (Iced)
- [ ] Main window layout (sidebar + content area)
- [ ] Halving list sidebar widget
- [ ] Dashboard view (current epoch, price, mempool cards)
- [ ] Chart view (AVWAP canvas rendering)
- [ ] Detail view (halving info, performance metrics)
- [ ] Status bar (API connection indicators)
- [ ] Dark theme configuration

### API Layer
- [ ] BitStamp exchange data client
- [ ] Mempool.space blockchain data client
- [ ] Async fetch integration with Iced Commands/Subscriptions

### Database Layer (SQLite)
- [ ] Schema design (candles table, halvings table, cache table)
- [ ] SQLite integration (rusqlite or sqlx)
- [ ] Pre-compiled database bundling strategy
- [ ] Data migration logic (seed → user data directory)

### Chart Engine
- [ ] Iced Canvas custom widget for price charts
- [ ] AVWAP calculation logic
- [ ] Chart interaction (zoom, timeframe selection)

### Build & Deployment
- [ ] Verify Rust edition compatibility (2024 vs 2021)
- [ ] Test suite setup
- [ ] CI configuration
- [ ] Binary release packaging

## Known Issues
- `Cargo.toml` uses `edition = "2024"` which may not be compatible with Iced 0.14.0 — may need to downgrade to `edition = "2021"`
- No dependencies added yet beyond Iced (need rusqlite/sqlx, reqwest, serde, chrono)
- No test infrastructure in place

## Project Decision Log
| Date | Decision | Rationale |
|---|---|---|
| 2026-07-17 | Rust + Iced tech stack | Desktop app with native performance; Iced is the leading Rust GUI framework |
| 2026-07-17 | BitStamp + Mempool.space data sources | BitStamp provides longest BTC exchange history; Mempool.space is the standard for blockchain data |
| 2026-07-17 | SQLite for local storage | Simple, zero-config, good for caching and pre-compiled data |
| 2026-07-17 | Iced Sandbox vs Application | Application trait required for async API calls via Command/Subscription |
| 2026-07-17 | Memory Bank initialized | Ensures session-persistent context across all development work |