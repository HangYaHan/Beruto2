use std::collections::HashMap;

use crate::data::PriceData;

use super::portfolio::Portfolio;
use super::strategy::{Action, Strategy};
use super::types::{BacktestOutput, NavPoint};

const DIVIDENDS_CSV: &str = "data/etf_dividends.csv";

pub fn load_dividend_map() -> HashMap<String, Vec<(String, f64)>> {
    let mut map: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let mut reader = csv::Reader::from_path(DIVIDENDS_CSV)
        .expect("Cannot open data/etf_dividends.csv");

    for result in reader.records() {
        let record = result.expect("Bad dividend CSV row");
        let code = record[0].to_string();
        let ex_date = record[1].to_string();
        let amount: f64 = record[2].parse().expect("Bad dividend amount");
        map.entry(ex_date).or_default().push((code, amount));
    }

    map
}

pub fn run_backtest(
    prices: &PriceData,
    strategy: &mut dyn Strategy,
) -> BacktestOutput {
    let targets = strategy.targets().clone();
    let mut portfolio = Portfolio::new(100_000.0);
    let div_map = load_dividend_map();

    // Initial allocation on first day
    let first_prices = prices.prices_at(0);
    let mut trade_events = portfolio.initial_allocate(&first_prices, &targets);
    // Set initial trade dates
    let first_date_str = prices.dates[0].format("%Y-%m-%d").to_string();
    for ev in &mut trade_events {
        ev.date = first_date_str.clone();
    }

    let mut nav_curve: Vec<NavPoint> = Vec::new();
    let mut dividend_events = Vec::new();
    let mut rebalance_events = Vec::new();

    nav_curve.push(NavPoint {
        date: first_date_str.clone(),
        value: portfolio.total_value(&first_prices),
    });

    let date_count = prices.len();

    for i in 1..date_count {
        let date = prices.dates[i];
        let date_str = date.format("%Y-%m-%d").to_string();
        let day_prices = prices.prices_at(i);

        // Apply dividends
        if let Some(divs) = div_map.get(&date_str) {
            for (code, amount) in divs {
                if let Some(ev) = portfolio.receive_dividend(code, *amount, &date_str) {
                    dividend_events.push(ev);
                }
            }
        }

        // Check strategy
        match strategy.step(date, i, &prices.dates, &day_prices, &portfolio) {
            Action::Rebalance(t) => {
                if let Some(ev) = portfolio.rebalance_to_targets(&day_prices, &t, &date_str) {
                    rebalance_events.push(ev);
                }
            }
            Action::None => {}
        }

        let val = portfolio.total_value(&day_prices);
        nav_curve.push(NavPoint {
            date: date_str,
            value: val,
        });
    }

    let final_value = nav_curve.last().map(|n| n.value).unwrap_or(0.0);
    let total_return = final_value / portfolio.initial_capital - 1.0;

    let total_div: f64 = dividend_events.iter().map(|e| e.total_income).sum();
    let dividend_yield = total_div / portfolio.initial_capital;

    let mut div_breakdown: HashMap<String, f64> = HashMap::new();
    for ev in &dividend_events {
        *div_breakdown.entry(ev.code.clone()).or_insert(0.0) += ev.total_income;
    }

    BacktestOutput {
        strategy_name: strategy.name().to_string(),
        start_date: prices.dates[0].format("%Y-%m-%d").to_string(),
        end_date: prices.dates.last().unwrap().format("%Y-%m-%d").to_string(),
        initial_capital: portfolio.initial_capital,
        parameters: strategy.parameters(),
        nav_curve,
        trade_events,
        dividend_events,
        rebalance_events,
        total_return,
        dividend_yield,
        dividend_breakdown: div_breakdown,
        final_value,
    }
}
