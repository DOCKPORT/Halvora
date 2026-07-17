# System Patterns: Halvora

## Architecture Overview
Halvora follows the **Iced MVU (Model-View-Update)** architecture pattern. The application is structured as a single-window desktop app with modular backend data sources.

```
┌─────────────────────────────────────────────────────────┐
│                      Iced Application                     │
│  ┌───────────────────────────────────────────────────┐  │
│  │                    Main Window                     │  │
│  │  ┌──────────────┐  ┌──────────────────────────┐   │  │
│  │  │   Sidebar    │  │      Content Area        │   │  │
│  │  │ ─────────── │  │ ┌──────────────────────┐ │   │  │
│  │  │ • Halving 1 │  │ │   Dashboard View    │ │   │  │
│  │  │ • Halving 2 │  │ │                      │ │   │  │
│  │  │ • Halving 3 │  │ │   Chart View        │ │   │  │
│  │  │ • ...       │  │ │                      │ │   │  │
│  │  │ • Halving 32│  │ │   Detail View       │ │   │  │
│  │  └──────────────┘  │ └──────────────────────┘ │   │  │
│  │                     └──────────────────────────┘   │  │
│  └───────────────────────────────────────────────────┘  │
│                          │                               │
│                          ▼                               │
│  ┌───────────────────────────────────────────────────┐  │
│  │                    Update Loop                      │  │
│  │  Message → Mutation → Render                       │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐   ┌───────────────────┐
│  BitStamp API    │   │  Mempool.space API│
│  (Exchange Data) │   │  (Blockchain Data)│
└────────┬────────┘   └────────┬──────────┘
         │                     │
         ▼                     ▼
┌──────────────────────────────────────────────┐
│              SQLite Database                  │
│  • Daily OHLCV candles                       │
│  • Halving schedule & block heights          │
│  • Cached API responses                      │
└──────────────────────────────────────────────┘
```

## Key Architectural Patterns

### 1. Iced MVU Pattern
- **Model**: Central application state containing halving data, exchange rates, UI state
- **View**: Pure functions that render Iced widgets based on Model state
- **Update**: Message-driven state mutations triggered by user interaction or API responses

### 2. Modular Data Source Pattern
- Each API source (BitStamp, Mempool) is an independent module
- Modules expose a consistent trait/interface for data fetching
- SQLite acts as the unified persistence layer across all data sources

### 3. Pre-compiled Database Pattern
- A seed database ships with the binary
- On first run, the seed database is copied to the user data directory
- Subsequent runs append new data to the local copy

## Component Tree (Iced Widgets)
```
HalvoraApp (Application)
├── MainWindow (Container)
│   ├── Sidebar
│   │   ├── HalvingList (scrollable)
│   │   │   └── HalvingListItem (×32)
│   │   └── StatusBar (API connection indicators)
│   └── ContentArea
│       ├── DashboardView
│       │   ├── CurrentEpochCard
│       │   ├── PriceCard (live BTC price)
│       │   └── MempoolCard (fee estimates)
│       ├── ChartView
│       │   ├── Canvas (AVWAP chart rendering)
│       │   └── ChartControls (timeframe, zoom)
│       └── DetailView
│           ├── HalvingInfo (block height, date, reward)
│           └── PerformanceMetrics (ROI, P/L, etc.)
```

## Critical Implementation Paths
1. **Main window layout** — Iced Column/Row/Container composition for sidebar + content split
2. **Widget messaging** — Custom enum messages for sidebar selection, API fetch triggers, chart interactions
3. **Canvas chart rendering** — Custom Iced Widget/Canvas drawing for AVWAP charts
4. **SQLite integration** — `rusqlite` or `sqlx` for database operations within the Iced update loop
5. **API async fetching** — Iced's `Command` + `Subscription` for non-blocking API calls