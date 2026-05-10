pub mod benchmarks;
pub mod metrics;
pub mod report;

use std::collections::HashMap;
use std::fs;

use crate::backtest::types::{BacktestOutput, NavPoint};
use crate::data::PriceData;

use self::benchmarks::BenchmarkData;
use self::metrics::*;
use self::report::{generate_html, MonthlyCell};

pub fn evaluate(output: &BacktestOutput, prices: &PriceData) -> String {
    let rf_daily = daily_risk_free();
    let nav_values: Vec<f64> = output.nav_curve.iter().map(|n| n.value).collect();
    let s_ret = daily_returns(&nav_values);
    let days = s_ret.len() as i64;

    // Benchmark
    let benchmark = BenchmarkData::load("data/benchmark.csv");
    let (b_ret, _) = benchmark.align_with(&prices.dates);

    let b_total_ret = (benchmark.closes.last().unwrap_or(&1.0) / benchmark.closes.first().unwrap_or(&1.0)) - 1.0;

    let b = beta(&s_ret, &b_ret);

    let total_ret_clean = output.total_return; // already includes fees + dividends

    let metrics = StrategyMetrics {
        total_return: total_ret_clean,
        annualized_return: annualized_return(total_ret_clean, days),
        max_drawdown: 0.0,
        max_drawdown_start: String::new(),
        max_drawdown_end: String::new(),
        max_drawdown_days: 0,
        sharpe: sharpe(&s_ret, rf_daily),
        sortino: sortino(&s_ret, rf_daily),
        calmar: 0.0,
        beta: b,
        alpha: alpha(total_ret_clean, b, b_total_ret, RISK_FREE_RATE, days),
        risk_free_rate: RISK_FREE_RATE,
    };

    let dd_info = max_drawdown(&output.nav_curve);
    let calmar_val = calmar(total_ret_clean, dd_info.max_drawdown);

    let metrics = StrategyMetrics {
        max_drawdown: dd_info.max_drawdown,
        max_drawdown_start: dd_info.start_date.clone(),
        max_drawdown_end: dd_info.end_date.clone(),
        max_drawdown_days: dd_info.duration_days,
        calmar: calmar_val,
        ..metrics
    };

    // Benchmark NAV normalized to strategy start
    let bench_nav: Vec<NavPoint> = benchmark
        .dates
        .iter()
        .zip(benchmark.closes.iter())
        .map(|(d, c)| NavPoint {
            date: d.format("%Y-%m-%d").to_string(),
            value: *c / benchmark.closes[0] * output.nav_curve[0].value,
        })
        .collect();

    // Per-ETF returns
    let mut etf_returns: HashMap<String, (f64, f64)> = HashMap::new();
    for code in &prices.codes {
        if let Some(closes) = prices.closes.get(code) {
            let ret = closes.last().unwrap_or(&1.0) / closes.first().unwrap_or(&1.0) - 1.0;
            let div = output.dividend_breakdown.get(code).copied().unwrap_or(0.0);
            let initial_price = closes[0];
            let shares = output.trade_events.iter()
                .find(|e| e.code == *code)
                .map(|e| e.shares)
                .unwrap_or(1.0);
            let div_ret = div / (shares * initial_price);
            let total = ret + div_ret;
            etf_returns.insert(code.clone(), (total, 0.0));
        }
    }

    // Monthly heatmap
    let monthly = build_monthly_heatmap(&output.nav_curve);

    // Rolling metrics
    let s_dates: Vec<String> = output.nav_curve.iter().map(|n| n.date.clone()).collect();
    let rolling = rolling_metrics(&s_ret, &b_ret, &s_dates, ROLLING_WINDOW, rf_daily);

    let html = generate_html(
        &metrics,
        &output.nav_curve,
        &bench_nav,
        &output.nav_curve,
        &dd_info,
        &etf_returns,
        &output.rebalance_events,
        &output.trade_events,
        &output.dividend_events,
        &rolling,
        &monthly,
    );

    fs::create_dir_all("result").ok();
    fs::write("result/report.html", &html).expect("Cannot write report.html");

    html
}

fn build_monthly_heatmap(nav: &[NavPoint]) -> Vec<MonthlyCell> {
    let mut cells = Vec::new();
    for i in 1..nav.len() {
        let prev_val = nav[i - 1].value;
        let cur_val = nav[i].value;
        let ret = (cur_val / prev_val) - 1.0;

        let d = &nav[i].date;
        if let Some(year) = d[..4].parse::<i32>().ok() {
            if let Some(month) = d[5..7].parse::<u32>().ok() {
                let _key = (year, month);
                if let Some(cell) = cells.iter_mut().find(|c: &&mut MonthlyCell| c.year == year && c.month == month) {
                    cell.value += ret;
                } else {
                    cells.push(MonthlyCell { year, month, value: ret });
                }
            }
        }
    }
    cells
}
