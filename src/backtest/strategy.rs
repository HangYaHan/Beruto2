use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};

use super::portfolio::Portfolio;

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

// ── Quarterly Rebalance Strategy ──────────────────────────────────────

pub struct QuarterlyRebalanceStrategy {
    targets: HashMap<String, f64>,
    threshold: f64,
    cash_floor: f64,
    quarter_months: Vec<u32>,
}

impl QuarterlyRebalanceStrategy {
    pub fn new(
        ndx_code: &str,
        div_codes: &[&str],
        gold_code: &str,
        threshold: f64,
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
            targets,
            threshold,
            cash_floor: 0.05,
            quarter_months: vec![3, 6, 9, 12],
        }
    }
}

impl Strategy for QuarterlyRebalanceStrategy {
    fn name(&self) -> &str {
        "Quarterly Rebalance"
    }

    fn parameters(&self) -> HashMap<String, String> {
        let mut p = HashMap::new();
        p.insert("threshold".to_string(), format!("{}", self.threshold));
        p.insert("cash_floor".to_string(), format!("{}", self.cash_floor));
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
        let month = date.month();

        if !self.quarter_months.contains(&month) {
            return Action::None;
        }

        let is_last_of_month = date_index + 1 >= dates.len()
            || dates[date_index + 1].month() != month;

        if !is_last_of_month {
            return Action::None;
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
            Action::Rebalance(self.targets.clone())
        } else {
            Action::None
        }
    }

    fn targets(&self) -> &HashMap<String, f64> {
        &self.targets
    }
}
