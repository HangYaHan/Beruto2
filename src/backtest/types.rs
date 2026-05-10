use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub date: String,
    pub code: String,
    pub action: String,
    pub shares: f64,
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendEvent {
    pub date: String,
    pub code: String,
    pub shares: f64,
    pub amount_per_share: f64,
    pub total_income: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceEvent {
    pub date: String,
    pub portfolio_value: f64,
    pub weights_before: HashMap<String, f64>,
    pub weights_after: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavPoint {
    pub date: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestOutput {
    pub strategy_name: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub parameters: HashMap<String, String>,

    pub nav_curve: Vec<NavPoint>,

    pub trade_events: Vec<TradeEvent>,
    pub dividend_events: Vec<DividendEvent>,
    pub rebalance_events: Vec<RebalanceEvent>,

    pub total_return: f64,
    pub dividend_yield: f64,
    pub dividend_breakdown: HashMap<String, f64>,
    pub final_value: f64,
}
