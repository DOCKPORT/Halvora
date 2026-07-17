# Project Brief: Halvora

## Project Overview
Halvora is a desktop application built in Rust that tracks all 32 Bitcoin halvings and their price action. It provides block height precision, historical chart analysis, and performance metrics across every halving epoch — from genesis to the final Bitcoin subsidy.

## Core Requirements
- Track all 32 Bitcoin halving events with block-height precision
- Display BTC exchange data (daily candles from ~2012 onward)
- Fetch and display mempool/blockchain data from Mempool.space
- Store historical data locally in SQLite
- Provide chart analysis with AVWAP anchored from halving dates
- Calculate halving period P/L and return metrics
- Auto-fetch and persist exchange data
- Ship with a pre-compiled exchange database to avoid long initial sync

## Tech Stack
| Component | Technology |
|---|---|
| Language | Rust (edition 2024) |
| GUI Framework | Iced GUI 0.14.0 |
| Exchange Data | BitStamp API (by Robinhood) |
| Blockchain Data | Mempool.space API |
| Local Database | SQLite |

## Project Structure
```
halvora/
├── Cargo.toml
├── memory-bank/             # Session-persistent documentation
├── Halvora_Logo/
├── src/
│   ├── main.rs
│   └── Modules/
│       ├── API/
│       │   ├── BitStamp/    # Exchange data API client
│       │   └── Mempool/     # Blockchain data API client
│       └── UI/
│           └── mainwindow/  # Iced GUI main window
```

## Project Status
- Initial scaffolding complete (Cargo project, module directories)
- Iced 0.14.0 dependency added
- No implementation code written yet
- First build target: Main window UI with Iced