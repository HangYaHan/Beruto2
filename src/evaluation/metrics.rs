use serde::Serialize;

use crate::backtest::types::NavPoint;

pub const RISK_FREE_RATE: f64 = 0.02;
pub const ROLLING_WINDOW: usize = 60;

#[derive(Debug, Clone, Serialize)]
pub struct DrawdownInfo {
    pub max_drawdown: f64,
    pub start_date: String,
    pub end_date: String,
    pub duration_days: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RollingPoint {
    pub date: String,
    pub sharpe: f64,
    pub sortino: f64,
    pub beta: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub max_drawdown: f64,
    pub max_drawdown_start: String,
    pub max_drawdown_end: String,
    pub max_drawdown_days: i64,
    pub sharpe: f64,
    pub sortino: f64,
    pub calmar: f64,
    pub beta: f64,
    pub alpha: f64,
    pub risk_free_rate: f64,
}

pub fn daily_returns(values: &[f64]) -> Vec<f64> {
    values.windows(2).map(|w| (w[1] / w[0]) - 1.0).collect()
}

pub fn daily_risk_free() -> f64 {
    (1.0 + RISK_FREE_RATE).powf(1.0 / 365.0) - 1.0
}

pub fn annualized_return(total_ret: f64, days: i64) -> f64 {
    (1.0 + total_ret).powf(365.0 / days as f64) - 1.0
}

pub fn max_drawdown(nav: &[NavPoint]) -> DrawdownInfo {
    let mut peak = nav[0].value;
    let mut peak_idx = 0;
    let mut max_dd = 0.0;
    let mut dd_start = 0;
    let mut dd_end = 0;

    for (i, point) in nav.iter().enumerate() {
        if point.value > peak {
            peak = point.value;
            peak_idx = i;
        }
        let dd = (peak - point.value) / peak;
        if dd > max_dd {
            max_dd = dd;
            dd_start = peak_idx;
            dd_end = i;
        }
    }

    DrawdownInfo {
        max_drawdown: max_dd,
        start_date: nav[dd_start].date.clone(),
        end_date: nav[dd_end].date.clone(),
        duration_days: (dd_end - dd_start) as i64,
    }
}

pub fn beta(s_ret: &[f64], b_ret: &[f64]) -> f64 {
    let n = s_ret.len().min(b_ret.len());
    if n < 2 {
        return 0.0;
    }
    let s_mean = s_ret[..n].iter().sum::<f64>() / n as f64;
    let b_mean = b_ret[..n].iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    let mut var = 0.0;
    for i in 0..n {
        let sd = s_ret[i] - s_mean;
        let bd = b_ret[i] - b_mean;
        cov += sd * bd;
        var += bd * bd;
    }
    if var == 0.0 {
        0.0
    } else {
        cov / var
    }
}

pub fn alpha(
    s_total_ret: f64,
    s_beta: f64,
    b_total_ret: f64,
    rf: f64,
    days: i64,
) -> f64 {
    let s_annual = annualized_return(s_total_ret, days);
    let b_annual = annualized_return(b_total_ret, days);
    s_annual - (rf + s_beta * (b_annual - rf))
}

pub fn sharpe(daily_ret: &[f64], rf_daily: f64) -> f64 {
    let excess: Vec<f64> = daily_ret.iter().map(|r| r - rf_daily).collect();
    let n = excess.len();
    if n < 2 {
        return 0.0;
    }
    let mean = excess.iter().sum::<f64>() / n as f64;
    let var = excess.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (n as f64 - 1.0);
    let daily_sharpe = mean / var.sqrt();
    daily_sharpe * (365.0_f64).sqrt()
}

pub fn sortino(daily_ret: &[f64], rf_daily: f64) -> f64 {
    let excess: Vec<f64> = daily_ret.iter().map(|r| r - rf_daily).collect();
    let negative: Vec<f64> = excess.iter().filter(|&&e| e < 0.0).copied().collect();
    let n_neg = negative.len();
    if n_neg < 2 {
        return 0.0;
    }
    let mean = excess.iter().sum::<f64>() / excess.len() as f64;
    let downside_var = negative.iter().map(|&e| e.powi(2)).sum::<f64>() / n_neg as f64;
    let daily_sortino = mean / downside_var.sqrt();
    daily_sortino * (365.0_f64).sqrt()
}

pub fn calmar(total_ret: f64, max_dd: f64) -> f64 {
    if max_dd.abs() < 1e-10 {
        0.0
    } else {
        total_ret / max_dd.abs()
    }
}

pub fn rolling_metrics(
    s_ret: &[f64],
    b_ret: &[f64],
    dates: &[String],
    window: usize,
    rf_daily: f64,
) -> Vec<RollingPoint> {
    let n = s_ret.len().min(b_ret.len());
    let mut points = Vec::new();

    for i in window..n {
        let s_win = &s_ret[i - window..i];
        let b_win = &b_ret[i - window..i];
        let b = beta(s_win, b_win);
        let sh = sharpe(s_win, rf_daily);
        let so = sortino(s_win, rf_daily);
        points.push(RollingPoint {
            date: dates[i].clone(),
            sharpe: sh,
            sortino: so,
            beta: b,
        });
    }

    points
}
