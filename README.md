# Beruto

ETF backtesting toolkit for Chinese A-share market.

## Overview

- **Data fetching** — Python scripts that pull daily K-line, dividend records, and benchmark data via akshare (东方财富)
- **Backtest engine** — Rust library for running trading strategies against historical data
- **Evaluation** — Rust module that computes risk metrics (Beta, Alpha, Sharpe, Sortino, Calmar) and generates an interactive HTML report with Chart.js
- **Visualization** — Python candlestick chart (day/week/month)

## Directory structure

```
scripts/                 Python data-fetching scripts (akshare)
  fetch_etf.py           Download daily K-line -> data/*.csv
  fetch_dividends.py     Fetch ETF dividend records -> data/etf_dividends.csv
  fetch_benchmark.py     Download SSE Composite Index -> data/benchmark.csv
src/
  data/mod.rs            CSV loading, date parsing, PriceData struct
  backtest/
    engine.rs            Main backtest loop
    portfolio.rs         Holdings, cash, NAV, weights, rebalancing (with trade fee)
    types.rs             BacktestOutput, NavPoint, event structs (serde JSON)
    strategy/
      mod.rs             Strategy trait + Action enum + factory + re-exports
      rebalance.rs       RebalanceStrategy implementation (configurable)
      strategies.json    Registry of all available strategies
  evaluation/
    mod.rs               Orchestrator: metrics -> report
    metrics.rs           Beta, Alpha, Sharpe, Sortino, Calmar, annualized returns
    benchmarks.rs        Load SSE Composite Index, align with strategy dates
    report.rs            Chart.js HTML template (7 chart types)
data/                    Read-only CSV raw data + cached dividends + benchmark
result/                  Per-run subfolders: backtest_output.json, report.html, summary.md
```

## Quick start

### Prerequisites

- Rust toolchain (`rustup`)
- Python 3.10+ with `pip install -r requirements.txt`

### Fetch data

```bash
python scripts/fetch_etf.py          # Download daily K-line for all ETFs
python scripts/fetch_dividends.py    # Refresh dividend cache (slow — API paginates)
python scripts/fetch_benchmark.py    # Download SSE Composite Index
```

### Run backtest + evaluation

```bash
cargo run
```

Runs all registered strategies and outputs per-run subfolders:

```
result/2026-05-12_qrt_cond/
  ├── backtest_output.json   # full backtest trace (NAV, trades, dividends, rebalances)
  ├── report.html            # interactive evaluation report (open in browser)
  └── summary.md             # strategy name, parameters, symbols
```

### Visualize (Python)

```bash
python visualize_etf.py 159941 -p w      # Interactive weekly K-line chart
python visualize_etf.py 518880 -p m -s   # Save monthly chart to result/
```

## Data assets

| File | Content |
|------|---------|
| `data/159941.csv` | Nasdaq ETF daily K-line |
| `data/159581.csv` | Dividend ETF (万家) daily K-line |
| `data/561580.csv` | Dividend ETF (华泰柏瑞) daily K-line |
| `data/563020.csv` | Dividend ETF (易方达) daily K-line |
| `data/518880.csv` | Gold ETF daily K-line |
| `data/etf_dividends.csv` | Dividend payout records (cached) |
| `data/benchmark.csv` | Shanghai Composite Index (上证综指) |

## Strategies

All strategies share target allocation: **NDX 40%, DIV 40% (3 ETFs equally), GOLD 10%, CASH 10%**.
Trade fee: **0.025%** per transaction.

| ID | Name | Check Period | Condition | Emergency Trigger |
|----|------|-------------|-----------|-------------------|
| `qrt_cond` | Quarterly Conditional | Quarter-end | Weight deviation >20% or cash <5% | — |
| `qrt_force` | Quarterly Force | Quarter-end | Always rebalance | — |
| `mth_cond` | Monthly Conditional | Month-end | Weight deviation >20% or cash <5% | — |
| `mth_force` | Monthly Force | Month-end | Always rebalance | — |
| `qrt_cond_trig` | Quarterly Conditional + Trigger | Quarter-end | Weight deviation >20% or cash <5% | 10% price move |
| `qrt_force_trig` | Quarterly Force + Trigger | Quarter-end | Always rebalance | 10% price move |
| `mth_cond_trig` | Monthly Conditional + Trigger | Month-end | Weight deviation >20% or cash <5% | 10% price move |
| `mth_force_trig` | Monthly Force + Trigger | Month-end | Always rebalance | 10% price move |

**Emergency trigger**: if any held ETF's price moves ±10% from its last rebalance reference price, an immediate rebalance fires regardless of scheduled check date.

### Adding a strategy

Add an entry to `src/backtest/strategy/strategies.json`:

```json
{
  "id": "my_strategy",
  "name": "My Strategy",
  "description": "Description of what it does.",
  "check_months": [3, 6, 9, 12],
  "force_rebalance": false,
  "threshold": 0.20,
  "cash_floor": 0.05,
  "emergency_threshold": null
}
```

`cargo run` will automatically pick up the new entry.

## Evaluation metrics

Each `result/*/report.html` includes:

- NAV curve vs benchmark (SSE Composite Index)
- Rolling Sharpe / Sortino / Beta (60-day window)
- Monthly return heatmap
- Drawdown chart
- Per-ETF return breakdown
- Rebalance event timeline

Metrics computed: Total Return, Annualized Return, Max Drawdown, Sharpe, Sortino, Calmar, Beta, Alpha.

## License

MIT
