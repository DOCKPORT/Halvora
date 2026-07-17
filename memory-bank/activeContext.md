# Active Context: Halvora

## Current Work Focus
Building the main window UI using Iced 0.14.0. This is the first implementation task after project scaffolding.

## Recent Changes
- Initialized Cargo project with Iced dependency
- Created project directory structure (API modules, UI modules)
- Set up GitHub repository (origin: https://github.com/DOCKPORT/Halvora.git)
- Created README.md with project overview
- Initialized Memory Bank with all 6 core files

## Next Steps
1. **Main window layout** (`src/Modules/UI/mainwindow/`):
   - Define the Iced Application struct (Model)
   - Implement the View function (sidebar + content area layout)
   - Implement the Update function (message handling)
   - Create halving list sidebar widget
   - Create placeholder content area views (Dashboard, Chart, Detail)

2. **Dependency additions** (`Cargo.toml`):
   - Add `rusqlite` (or `sqlx`) for SQLite
   - Add `reqwest` for HTTP API calls
   - Add `serde`/`serde_json` for JSON deserialization
   - Add `chrono` for date/time

3. **API module implementation** (separate task):
   - BitStamp exchange data client
   - Mempool.space blockchain data client

4. **SQLite database layer** (separate task):
   - Schema design for candles, halvings, cache
   - Pre-compiled database bundling strategy

5. **Chart rendering** (future task):
   - Iced Canvas widget for AVWAP charts
   - Chart controls and interaction

## Active Decisions & Considerations

### Edition 2024 Compatibility
Cargo.toml specifies `edition = "2024"` — this is very new and may cause issues. If compilation fails, we should switch to `edition = "2021"`.

### Main Window Layout Strategy
- Use Iced's `PaneGrid` for resizable sidebar/content split, OR
- Use a simpler `Container` + `Row` with fixed-width sidebar (menu-driven navigation)
- Decision: Start with simple Row layout, migrate to PaneGrid if resizable panels are needed

### State Management
- All halving data will live in the Iced Model struct
- API data will be fetched via async Commands and stored in the Model
- View will render based on current Model state (no separate state management library needed)

## Important Patterns & Preferences
- Follow Iced's MVU pattern strictly
- Keep UI components modular (one file per major view/widget)
- Use descriptive enum messages for all user interactions
- Dark theme (standard for crypto tools)