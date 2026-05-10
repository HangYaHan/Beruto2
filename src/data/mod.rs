use chrono::NaiveDate;
use std::collections::HashMap;

pub struct PriceData {
    pub dates: Vec<NaiveDate>,
    pub closes: HashMap<String, Vec<f64>>,
    pub codes: Vec<String>,
}

impl PriceData {
    pub fn load(data_dir: &str, codes: &[&str]) -> Self {
        let mut dates: Option<Vec<NaiveDate>> = None;
        let mut closes = HashMap::new();
        let mut code_strings = Vec::new();

        for code in codes {
            let path = format!("{}/{}.csv", data_dir, code);
            let (d, c) = read_csv_close(&path);
            if let Some(ref existing) = dates {
                if d.len() != existing.len() {
                    eprintln!("Warning: {code} has {} rows, expected {}", d.len(), existing.len());
                }
            } else {
                dates = Some(d);
            }
            closes.insert(code.to_string(), c);
            code_strings.push(code.to_string());
        }

        Self {
            dates: dates.unwrap_or_default(),
            closes,
            codes: code_strings,
        }
    }

    pub fn prices_at(&self, index: usize) -> HashMap<String, f64> {
        self.closes
            .iter()
            .map(|(k, v)| (k.clone(), v[index]))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.dates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dates.is_empty()
    }
}

fn read_csv_close(path: &str) -> (Vec<NaiveDate>, Vec<f64>) {
    let mut reader = csv::Reader::from_path(path).expect(&format!("Cannot open {path}"));
    let mut dates = Vec::new();
    let mut closes = Vec::new();
    for result in reader.records() {
        let record = result.expect("CSV parse error");
        let date = NaiveDate::parse_from_str(&record[0], "%Y-%m-%d")
            .expect(&format!("Bad date: {}", &record[0]));
        let close: f64 = record[4]
            .parse()
            .expect(&format!("Bad close price: {}", &record[4]));
        dates.push(date);
        closes.push(close);
    }
    (dates, closes)
}
