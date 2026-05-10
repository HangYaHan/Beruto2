## Project

ETF data toolkit for Chinese A-share market. Fetches daily K-line data for specified ETF codes via akshare (东方财富), stores as CSV, and provides candlestick chart visualization with day/week/month scales plus return-rate analysis.

## Agent Instructions

- Think step by step before taking any action. Reason through the problem, consider alternatives, and verify assumptions before writing code or executing commands.
- At the end of every response, output a brief summary in Chinese (中文) describing what was done.
- Follow the UNIX Philosophy: each component should do one thing and do it well. Keep modules, functions, and classes focused on a single responsibility.
- Prefer Rust for all new code. Use Rust as the primary language unless there is a compelling reason to use something else.

## Data Rules

- When implementing a strategy, only use symbols already present in `./data/` CSVs. If a required symbol has no data file in `./data/`, **abort** and ask the user — never fetch new data without permission.
- `./data/` is **read-only**. Only write to `./data/` when the user explicitly instructs you to do so.
- Symbol mapping:
  - `159941` = Nasdaq ETF (纳指)
  - `518880` = Gold ETF (黄金)
  - `159581`, `561580`, `563020` = Dividend ETFs (红利). When the user says "红利" or "dividend" without specifying codes, split equally among these three.
- All output (charts, CSVs, generated results) must go to `./result/`.
- ETF trading fee: **0.025%** (0.00025) per transaction, applied to all buys and sells.

## Structure

| Path | Purpose |
|------|---------|
| `scripts/fetch_etf.py` | Download daily K-line via akshare -> `data/*.csv` |
| `scripts/fetch_dividends.py` | Fetch ETF dividend records -> `data/etf_dividends.csv` (slow, use cache) |
| `src/data/mod.rs` | CSV price loading, `PriceData` struct, date parsing |
| `src/backtest/types.rs` | `BacktestOutput`, `NavPoint`, event structs (serde) |
| `src/backtest/portfolio.rs` | Holdings, cash, NAV, weights, rebalancing |
| `src/backtest/strategy.rs` | `Strategy` trait + `QuarterlyRebalanceStrategy` |
| `src/backtest/engine.rs` | Main backtest loop -> `result/backtest_output.json` |
| `src/evaluation/mod.rs` | Evaluator: metrics + HTML report generation |
| `src/evaluation/metrics.rs` | Beta, Alpha, Sharpe, Sortino, Calmar, annualized returns |
| `src/evaluation/benchmarks.rs` | Load SSE Composite Index, align with strategy dates |
| `src/evaluation/report.rs` | Chart.js HTML template (all 7 chart types) |
| `data/` | CSV raw data, `etf_dividends.csv` dividend cache, `benchmark.csv` SSE index |
| `result/` | Backtest JSON output, evaluation HTML report, charts |
| `visualize_etf.py` | Candlestick chart (d/w/m), argparse |

## Commands

```
# Data fetching (Python)
python scripts/fetch_etf.py                          # download/refresh all ETF data
python scripts/fetch_dividends.py                    # refresh dividend cache (slow)
python scripts/fetch_benchmark.py                    # download SSE Composite Index

# Backtest (Rust)
cargo run                                            # run backtest -> result/backtest_output.json
cargo build                                          # compile only
cargo test                                           # run tests

# Visualization (Python)
python visualize_etf.py <code> [-p d|w|m]            # interactive K-line chart
python visualize_etf.py <code> -p w -s               # save chart to result/<code>_w.png
```

## Dependencies

- Rust toolchain (`rustup`)
- Python: `pip install -r requirements.txt` (akshare, pandas, mplfinance, matplotlib)

## Quirks

- mplfinance has no stable release >=0.12.0; use `pip install mplfinance --pre`
- CJK fonts in mplfinance charts fail silently — English labels used to avoid glyph warnings
- Backtest output format is JSON: `result/backtest_output.json` (see `src/backtest/types.rs` for schema)
