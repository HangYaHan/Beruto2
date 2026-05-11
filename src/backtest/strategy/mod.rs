use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Deserialize;

use super::portfolio::Portfolio;

pub mod rebalance;
pub use rebalance::RebalanceStrategy;

pub enum Action {
    Rebalance(HashMap<String, f64>),
    None,
}

pub trait Strategy {
    fn name(&self) -> &str;
    fn parameters(&self) -> HashMap<String, String>;
    fn step(
        &mut self,
        date: NaiveDate,
        date_index: usize,
        dates: &[NaiveDate],
        prices: &HashMap<String, f64>,
        portfolio: &Portfolio,
    ) -> Action;
    fn targets(&self) -> &HashMap<String, f64>;
}

// ── Strategy registry from JSON ──

#[derive(Debug, Deserialize)]
struct StrategyConfig {
    id: String,
    name: String,
    #[allow(dead_code)]
    description: String,
    check_months: Vec<u32>,
    force_rebalance: bool,
    threshold: f64,
    cash_floor: f64,
    emergency_threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Registry {
    strategies: Vec<StrategyConfig>,
}

fn load_registry() -> Registry {
    let json = include_str!("strategies.json");
    serde_json::from_str(json).expect("Failed to parse strategies.json")
}

pub fn build_strategy(
    id: &str,
    ndx_code: &str,
    div_codes: &[&str],
    gold_code: &str,
) -> Option<RebalanceStrategy> {
    let registry = load_registry();
    let config = registry.strategies.iter().find(|s| s.id == id)?;

    Some(RebalanceStrategy::new(
        &config.id,
        &config.name,
        ndx_code,
        div_codes,
        gold_code,
        config.threshold,
        config.cash_floor,
        config.check_months.clone(),
        config.force_rebalance,
        config.emergency_threshold,
    ))
}

pub fn list_strategies() -> Vec<(String, String)> {
    load_registry()
        .strategies
        .iter()
        .map(|s| (s.id.clone(), s.name.clone()))
        .collect()
}
