# Product Context: Halvora

## Why Halvora Exists
Bitcoin halvings are the most important structural events in Bitcoin's monetary policy, yet there is no dedicated desktop tool that tracks all 32 halvings comprehensively. Existing solutions are scattered across web dashboards, spreadsheets, and fragmented data sources. Halvora fills this gap with a single-pane desktop application that provides complete halving lifecycle tracking.

## Problems It Solves
- **Fragmented data**: Halving dates, block heights, price data, and blockchain stats are spread across multiple services
- **No 32-halving view**: Most tools only track the next halving; Halvora tracks all 32 epochs from genesis to the final subsidy
- **No anchored chart analysis**: AVWAP and performance metrics need to be anchored to specific halving dates for meaningful cycle analysis
- **Manual data collection**: Users currently need to aggregate exchange data, mempool stats, and halving schedules manually

## How It Should Work
1. **Launch**: Application opens to a dashboard showing the current halving epoch status
2. **Halving Timeline**: A scrollable sidebar or timeline view showing all 32 halvings with block heights, dates, and reward amounts
3. **Chart View**: Selecting a halving epoch opens an AVWAP-anchored price chart for that cycle
4. **Performance Metrics**: Each epoch displays P/L, ROI, and other return metrics
5. **Live Data**: Automatic fetching of current exchange rates and mempool statistics
6. **Persistence**: Historical data is cached locally in SQLite; a pre-compiled database ships with the binary

## User Experience Goals
- Clean, dark-themed desktop interface (standard for crypto tools)
- Fast startup with pre-compiled database — no wait for initial sync
- Intuitive navigation between halving epochs
- Responsive charts that feel native (leveraging Iced's canvas/widget system)
- Status indicators showing data freshness (last API fetch time)