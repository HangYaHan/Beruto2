use chrono::NaiveDate;
use std::collections::HashMap;

pub struct BenchmarkData {
    pub dates: Vec<NaiveDate>,
    pub closes: Vec<f64>,
}

impl BenchmarkData {
    pub fn load(path: &str) -> Self {
        let mut reader = csv::Reader::from_path(path).expect("Cannot open benchmark CSV");
        let mut dates = Vec::new();
        let mut closes = Vec::new();
        for result in reader.records() {
            let record = result.expect("Bad benchmark CSV row");
            let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")
                .expect("Bad benchmark date");
            let close: f64 = record[1].parse().expect("Bad benchmark close");
            dates.push(date);
            closes.push(close);
        }
        Self { dates, closes }
    }

    pub fn align_with(
        &self,
        strategy_dates: &[NaiveDate],
    ) -> (Vec<f64>, Vec<f64>) {
        let date_to_close: HashMap<NaiveDate, f64> = self
            .dates
            .iter()
            .copied()
            .zip(self.closes.iter().copied())
            .collect();

        let mut bench_aligned = Vec::with_capacity(strategy_dates.len());
        let mut strategy_indices = Vec::new();

        for (i, d) in strategy_dates.iter().enumerate() {
            if let Some(&c) = date_to_close.get(d) {
                bench_aligned.push(c);
                strategy_indices.push(i);
            }
        }

        let bench_ret = daily_returns(&bench_aligned);
        (bench_ret, strategy_indices.into_iter().map(|x| x as f64).collect())
    }
}

pub fn daily_returns(prices: &[f64]) -> Vec<f64> {
    prices.windows(2).map(|w| (w[1] / w[0]) - 1.0).collect()
}
