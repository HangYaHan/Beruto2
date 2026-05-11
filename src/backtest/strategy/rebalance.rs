use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use crate::backtest::portfolio::Portfolio;

use super::{Action, Strategy};

pub struct RebalanceStrategy {
    pub id: String,
    pub name: String,
    targets: HashMap<String, f64>,
    threshold: f64,
    cash_floor: f64,
    check_months: Vec<u32>,
    force_rebalance: bool,
    emergency_threshold: Option<f64>,
    reference_prices: HashMap<String, f64>,
}

impl RebalanceStrategy {
    pub fn new(
        id: &str,
        name: &str,
        ndx_code: &str,
        div_codes: &[&str],
        gold_code: &str,
        threshold: f64,
        cash_floor: f64,
        check_months: Vec<u32>,
        force_rebalance: bool,
        emergency_threshold: Option<f64>,
    ) -> Self {
        let n = div_codes.len() as f64;
        let div_each = 0.40 / n;

        let mut targets = HashMap::new();
        targets.insert(ndx_code.to_string(), 0.40);
        for code in div_codes {
            targets.insert(code.to_string(), div_each);
        }
        targets.insert(gold_code.to_string(), 0.10);
        targets.insert("CASH".to_string(), 0.10);

        Self {
            id: id.to_string(),
            name: name.to_string(),
            targets,
            threshold,
            cash_floor,
            check_months,
            force_rebalance,
            emergency_threshold,
            reference_prices: HashMap::new(),
        }
    }

    fn update_reference_prices(&mut self, prices: &HashMap<String, f64>) {
        self.reference_prices.clear();
        for code in self.targets.keys() {
            if code != "CASH" {
                if let Some(price) = prices.get(code) {
                    self.reference_prices.insert(code.clone(), *price);
                }
            }
        }
    }
}

impl Strategy for RebalanceStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn parameters(&self) -> HashMap<String, String> {
        let mut p = HashMap::new();
        p.insert("id".to_string(), self.id.clone());
        p.insert("threshold".to_string(), format!("{}", self.threshold));
        p.insert("cash_floor".to_string(), format!("{}", self.cash_floor));
        p.insert("force_rebalance".to_string(), format!("{}", self.force_rebalance));
        p.insert(
            "check_months".to_string(),
            format!("{:?}", self.check_months),
        );
        if let Some(et) = self.emergency_threshold {
            p.insert("emergency_threshold".to_string(), format!("{}", et));
        }
        for (k, v) in &self.targets {
            p.insert(format!("target_{k}"), format!("{:.4}", v));
        }
        p
    }

    fn step(
        &mut self,
        date: NaiveDate,
        date_index: usize,
        dates: &[NaiveDate],
        prices: &HashMap<String, f64>,
        portfolio: &Portfolio,
    ) -> Action {
        if self.reference_prices.is_empty() {
            self.update_reference_prices(prices);
        }

        let month = date.month();
        let is_check_month = self.check_months.contains(&month);
        let is_last_of_month =
            date_index + 1 >= dates.len() || dates[date_index + 1].month() != month;

        if is_check_month && is_last_of_month {
            if self.force_rebalance {
                self.update_reference_prices(prices);
                return Action::Rebalance(self.targets.clone());
            }

            let w = portfolio.weights(prices);
            let mut triggered = false;

            if *w.get("CASH").unwrap_or(&0.0) < self.cash_floor {
                triggered = true;
            }

            for (code, target_pct) in &self.targets {
                if code == "CASH" {
                    continue;
                }
                let actual = w.get(code).copied().unwrap_or(0.0);
                let upper = target_pct * (1.0 + self.threshold);
                let lower = target_pct * (1.0 - self.threshold);
                if actual > upper || actual < lower {
                    triggered = true;
                }
            }

            if triggered {
                self.update_reference_prices(prices);
                return Action::Rebalance(self.targets.clone());
            }
        }

        if let Some(et) = self.emergency_threshold {
            for (code, ref_price) in &self.reference_prices {
                if let Some(current_price) = prices.get(code) {
                    let change = (current_price / ref_price - 1.0).abs();
                    if change > et {
                        self.update_reference_prices(prices);
                        return Action::Rebalance(self.targets.clone());
                    }
                }
            }
        }

        Action::None
    }

    fn targets(&self) -> &HashMap<String, f64> {
        &self.targets
    }
}
