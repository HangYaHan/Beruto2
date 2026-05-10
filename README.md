# Beruto

ETF backtesting toolkit for Chinese A-share market.

## Overview

- **Data fetching** — Python scripts that pull daily K-line, dividend records, and benchmark data via akshare (东方财富)
- **Backtest engine** — Rust library for running trading strategies against historical data
- **Evaluation** — Rust module that computes risk metrics (Beta, Alpha, Sharpe, Sortino, Calmar) and generates an interactive HTML report with Chart.js
- **Visualization** — Python candlestick chart (day/week/month)

## Directory structure

```
scripts/             Python data-fetching scripts (akshare)
  fetch_etf.py       Download daily K-line -> data/*.csv
  fetch_dividends.py Fetch ETF dividend records -> data/etf_dividends.csv
  fetch_benchmark.py Download SSE Composite Index -> data/benchmark.csv
src/
  data/mod.rs        CSV loading, date parsing, PriceData struct
  backtest/
    engine.rs        Main backtest loop -> result/backtest_output.json
    portfolio.rs     Holdings, cash, NAV, weights, rebalancing (with trade fee)
    strategy.rs      Strategy trait + QuarterlyRebalanceStrategy
    types.rs         BacktestOutput, NavPoint, event structs (serde JSON)
  evaluation/
    mod.rs           Orchestrator: metrics -> report
    metrics.rs       Beta, Alpha, Sharpe, Sortino, Calmar, annualized returns
    benchmarks.rs    Load SSE Composite Index, align with strategy dates
    report.rs        Chart.js HTML template (7 chart types)
data/                Read-only CSV raw data + cached dividends + benchmark
result/              Backtest JSON, evaluation HTML report, charts
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

Output:
- `result/backtest_output.json` — full backtest trace (NAV, trades, dividends, rebalances)
- `result/report.html` — interactive evaluation report (open in browser)

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

## Strategy: Quarterly Rebalance

Target allocation: NDX 40%, DIV 40% (3 ETFs equally), GOLD 10%, CASH 10%.

Rebalancing triggers on last trading day of each quarter when any asset weight deviates from target by >20% relative (or CASH drops below 5%).

Backtest period: 2025-01-02 ~ 2026-05-08. Initial capital: ¥100,000. Trade fee: 0.025%.

## Evaluation metrics

| Metric | Value |
|--------|-------|
| Total Return | +24.79% |
| Annual Return | +28.54% |
| Max Drawdown | 10.77% |
| Sharpe Ratio | 1.520 |
| Sortino Ratio | 1.286 |
| Calmar Ratio | 2.302 |
| Beta (vs SSE) | 0.623 |
| Alpha (annual) | +7.58% |

## License

MIT
