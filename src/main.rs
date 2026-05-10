use std::fs;

use beruto::backtest::engine::{load_dividend_map, run_backtest};
use beruto::backtest::strategy::QuarterlyRebalanceStrategy;
use beruto::data::PriceData;
use beruto::evaluation::evaluate;

fn main() {
    let codes = ["159941", "159581", "561580", "563020", "518880"];

    println!("Loading price data...");
    let prices = PriceData::load("data", &codes);
    println!("  {} codes, {} trading days", codes.len(), prices.len());

    println!("Loading dividend data...");
    let _div_map = load_dividend_map();

    println!("\nRunning backtest: Quarterly Rebalance Strategy");
    let mut strategy = QuarterlyRebalanceStrategy::new(
        "159941",
        &["159581", "561580", "563020"],
        "518880",
        0.20,
    );

    let output = run_backtest(&prices, &mut strategy);

    // Print summary
    println!();
    println!("============================================================");
    println!("  Quarterly Rebalance Strategy Backtest");
    println!("  {} ~ {}", output.start_date, output.end_date);
    println!("============================================================");
    println!();
    println!("  Strategy:  {}", output.strategy_name);
    println!("  Parameters:");
    for (k, v) in &output.parameters {
        println!("    {}: {}", k, v);
    }
    println!();
    println!("  Initial Capital:  {:.2} CNY", output.initial_capital);
    println!("  Final Value:       {:.2} CNY", output.final_value);
    println!("  Total Return:      {:+.2}%", output.total_return * 100.0);
    println!();
    println!("  Dividend Events:   {}", output.dividend_events.len());
    println!("  Total Dividend:    {:.2} CNY", output.dividend_events.iter().map(|e| e.total_income).sum::<f64>());
    println!("  Dividend Yield:    {:+.2}%", output.dividend_yield * 100.0);
    println!();
    println!("  --- Dividend by ETF ---");
    for (code, total) in &output.dividend_breakdown {
        println!("    {}: {:.2} CNY", code, total);
    }
    println!();
    println!("  Rebalance Events:  {}", output.rebalance_events.len());
    for ev in &output.rebalance_events {
        println!("    {}  value={:.2} CNY", ev.date, ev.portfolio_value);
    }

    // Save JSON output
    let json = serde_json::to_string_pretty(&output).expect("JSON serialization failed");
    fs::create_dir_all("result").ok();
    fs::write("result/backtest_output.json", &json).expect("Cannot write result/backtest_output.json");
    println!();
    println!("Saved backtest output to result/backtest_output.json");

    // Evaluate and generate HTML report
    println!("\nGenerating evaluation report...");
    let _html = evaluate(&output, &prices);
    println!("Saved evaluation report to result/report.html");
}
