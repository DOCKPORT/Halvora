<div align="center">
  <br/>
  <img src="Halvora_Logo/Halvora.png?v=2" alt="Halvora Logo" width="500"/>
  <br/><br/>
</div>

<p align="center">
  <strong>Halvora is a desktop application that tracks all 32 Bitcoin halvings and their price action.</strong>
  <br/>
  From block height precision to trading chart analysis across halving periods.
</p>

<br/>

---

## The 32 Halvings

Bitcoin's block reward halves 32 times until the 33rd halving reduces it to 0 satoshis, meaning the 32nd halving itself is the "1 satoshi" epoch — the final positive reward before mining becomes fee-dependent.

- **Complete tracking** of every halving event from genesis to the final Bitcoin subsidy
- **Block height precision** — exact block-level detection of halving triggers. Additional blockchain stats: mining difficulty, coins issued, and more.
- **Time estimates** — projections for future halving events
- **Chart analysis** — Price action line charts and utilizes AVWAP anchored from the halving dates.
- **Performance analysis** — Analyzing halving period P/L and return metrics across each cycle.

## Data Pipeline

- **Daily candles** — BTC exchange data stored locally, going back to ~2012
- **SQLite database** — local storage for fetch and retrieval of historical data
- **Automatic updates** — program handles data fetching and persistence
- **Pre-compiled database** — Exchange database is pre-compiled and integrated in repo and binary. This prevents the need for a full initial fetch.

## Tech Stack

| Component | Technology |
|---|---|
| Language | **Rust** |
| GUI Framework | **ICED GUI** |
| Exchange Data | **BitStamp** (by Robinhood) |
| Blockchain Data | **Mempool.space** |
| Local Database | **SQLite** |
| Official Font | **Geist Mono** |
