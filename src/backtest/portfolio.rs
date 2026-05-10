use std::collections::HashMap;

use crate::backtest::types::{DividendEvent, RebalanceEvent, TradeEvent};

pub const TRADE_FEE: f64 = 0.00025;

pub struct Portfolio {
    pub cash: f64,
    pub holdings: HashMap<String, f64>,
    pub initial_capital: f64,
}

impl Portfolio {
    pub fn new(capital: f64) -> Self {
        Self {
            cash: capital,
            holdings: HashMap::new(),
            initial_capital: capital,
        }
    }

    pub fn total_value(&self, prices: &HashMap<String, f64>) -> f64 {
        let mut v = self.cash;
        for (code, shares) in &self.holdings {
            if let Some(price) = prices.get(code) {
                v += shares * price;
            }
        }
        v
    }

    pub fn weights(&self, prices: &HashMap<String, f64>) -> HashMap<String, f64> {
        let tv = self.total_value(prices);
        let mut w = HashMap::new();
        if tv <= 0.0 {
            return w;
        }
        w.insert("CASH".to_string(), self.cash / tv);
        for (code, shares) in &self.holdings {
            if let Some(price) = prices.get(code) {
                w.insert(code.clone(), shares * price / tv);
            }
        }
        w
    }

    pub fn receive_dividend(
        &mut self,
        code: &str,
        amount_per_share: f64,
        date: &str,
    ) -> Option<DividendEvent> {
        if let Some(&shares) = self.holdings.get(code) {
            if shares > 0.0 {
                let income = shares * amount_per_share;
                self.cash += income;
                return Some(DividendEvent {
                    date: date.to_string(),
                    code: code.to_string(),
                    shares,
                    amount_per_share,
                    total_income: income,
                });
            }
        }
        None
    }

    pub fn buy(&mut self, code: &str, quantity: f64, price: f64) {
        let cost = quantity * price;
        let fee = cost * TRADE_FEE;
        self.cash -= cost + fee;
        *self.holdings.entry(code.to_string()).or_insert(0.0) += quantity;
    }

    pub fn sell(&mut self, code: &str, quantity: f64, price: f64) {
        let revenue = quantity * price;
        let fee = revenue * TRADE_FEE;
        self.cash += revenue - fee;
        if let Some(held) = self.holdings.get_mut(code) {
            *held -= quantity;
            if *held < 0.0 {
                *held = 0.0;
            }
        }
    }

    pub fn rebalance_to_targets(
        &mut self,
        prices: &HashMap<String, f64>,
        targets: &HashMap<String, f64>,
        date: &str,
    ) -> Option<RebalanceEvent> {
        let weights_before = self.weights(prices);

        // Liquidate all holdings (sell fee deducted)
        let mut pool = self.cash;
        for (code, shares) in &self.holdings {
            if let Some(price) = prices.get(code) {
                let revenue = shares * price;
                pool += revenue * (1.0 - TRADE_FEE);
            }
        }

        self.cash = pool;
        self.holdings.clear();

        // Re-allocate with buy fee: shares = gross_alloc / (price * (1 + TRADE_FEE))
        for (code, target_pct) in targets {
            if code == "CASH" {
                continue;
            }
            if let Some(price) = prices.get(code) {
                let gross_alloc = pool * target_pct;
                let shares = gross_alloc / (price * (1.0 + TRADE_FEE));
                let cost = shares * price;
                let fee = cost * TRADE_FEE;
                self.cash -= cost + fee;
                self.holdings.insert(code.clone(), shares);
            }
        }

        let weights_after = self.weights(prices);

        Some(RebalanceEvent {
            date: date.to_string(),
            portfolio_value: pool,
            weights_before,
            weights_after,
        })
    }

    pub fn initial_allocate(
        &mut self,
        prices: &HashMap<String, f64>,
        targets: &HashMap<String, f64>,
    ) -> Vec<TradeEvent> {
        let mut events = Vec::new();
        let capital = self.initial_capital;

        let cash_target = targets.get("CASH").copied().unwrap_or(0.0);
        self.cash = capital * cash_target;

        for (code, target_pct) in targets {
            if code == "CASH" {
                continue;
            }
            if let Some(&price) = prices.get(code) {
                let gross_alloc = capital * target_pct;
                let shares = gross_alloc / (price * (1.0 + TRADE_FEE));
                self.holdings.insert(code.clone(), shares);
                events.push(TradeEvent {
                    date: String::new(),
                    code: code.clone(),
                    action: "BUY".to_string(),
                    shares,
                    price,
                    amount: gross_alloc,
                });
            }
        }

        events
    }
}
