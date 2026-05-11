use std::fs;

use chrono::Local;

use beruto::backtest::engine::{load_dividend_map, run_backtest};
use beruto::backtest::strategy::{build_strategy, list_strategies};
use beruto::data::PriceData;
use beruto::evaluation::evaluate;

fn main() {
    let codes = ["159941", "159581", "561580", "563020", "518880"];

    println!("Loading price data...");
    let prices = PriceData::load("data", &codes);
    println!("  {} codes, {} trading days", codes.len(), prices.len());

    println!("Loading dividend data...");
    let _div_map = load_dividend_map();

    let strategies = list_strategies();

    for (id, name) in &strategies {
        println!();
        println!("============================================================");
        println!("  {}", name);
        println!("============================================================");

        let mut strategy = build_strategy(
            id,
            "159941",
            &["159581", "561580", "563020"],
            "518880",
        )
        .expect("Strategy not found in registry");

        let output = run_backtest(&prices, &mut strategy);

        println!("  {} ~ {}", output.start_date, output.end_date);
        println!("  Initial Capital:  {:.2} CNY", output.initial_capital);
        println!("  Final Value:       {:.2} CNY", output.final_value);
        println!("  Total Return:      {:+.2}%", output.total_return * 100.0);
        println!("  Rebalance Events:  {}", output.rebalance_events.len());
        println!(
            "  Dividend Events:   {}",
            output.dividend_events.len()
        );

        let today = Local::now().format("%Y-%m-%d").to_string();
        let folder = format!("result/{}_{}", today, id);
        fs::create_dir_all(&folder).expect("Cannot create result folder");

        let json =
            serde_json::to_string_pretty(&output).expect("JSON serialization failed");
        fs::write(format!("{}/backtest_output.json", folder), &json)
            .expect("Cannot write backtest_output.json");

        let _html = evaluate(&output, &prices, &folder);
        println!("  Saved to {}", folder);
    }

    println!();
    println!("All {} strategies complete.", strategies.len());
}
