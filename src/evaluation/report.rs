use std::collections::HashMap;

use crate::backtest::types::{DividendEvent, NavPoint, RebalanceEvent, TradeEvent};

use super::metrics::{RollingPoint, StrategyMetrics};

pub fn generate_html(
    metrics: &StrategyMetrics,
    nav_curve: &[NavPoint],
    bench_nav: &[NavPoint],
    drawdown_data: &[NavPoint],
    dd_info: &super::metrics::DrawdownInfo,
    etf_returns: &HashMap<String, (f64, f64)>,
    rebalance_events: &[RebalanceEvent],
    trade_events: &[TradeEvent],
    dividend_events: &[DividendEvent],
    rolling: &[RollingPoint],
    monthly_heatmap: &[MonthlyCell],
) -> String {
    let nav_json = nav_to_json(nav_curve, "nav");
    let bench_json = nav_to_json(bench_nav, "bench");
    let dd_json = dd_to_json(drawdown_data);
    let etf_json = etf_to_json(etf_returns);
    let rebalance_json = rebalance_to_json(rebalance_events, trade_events);
    let rolling_json = rolling_to_json(rolling);
    let div_json = div_to_json(dividend_events);
    let heatmap_rows = heatmap_to_html(monthly_heatmap);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Strategy Evaluation Report</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
<style>
* {{ margin:0; padding:0; box-sizing:border-box; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background:#f5f5f5; color:#333; }}
.container {{ max-width:1200px; margin:0 auto; padding:24px; }}
h1 {{ font-size:28px; margin-bottom:8px; }}
.subtitle {{ color:#666; margin-bottom:24px; }}
.card-grid {{ display:grid; grid-template-columns:repeat(3,1fr); gap:16px; margin-bottom:32px; }}
.card {{ background:#fff; border-radius:8px; padding:20px; box-shadow:0 1px 3px rgba(0,0,0,.08); }}
.card .label {{ font-size:13px; color:#888; text-transform:uppercase; letter-spacing:.5px; }}
.card .value {{ font-size:26px; font-weight:700; margin-top:4px; }}
.card .value.positive {{ color:#22c55e; }}
.card .value.negative {{ color:#ef4444; }}
.chart-section {{ background:#fff; border-radius:8px; padding:24px; margin-bottom:24px; box-shadow:0 1px 3px rgba(0,0,0,.08); }}
.chart-section h2 {{ font-size:18px; margin-bottom:16px; color:#444; }}
.chart-wrapper {{ position:relative; width:100%; }}
.chart-wrapper canvas {{ width:100% !important; }}
.heatmap {{ overflow-x:auto; }}
.heatmap table {{ border-collapse:collapse; width:100%; }}
.heatmap th, .heatmap td {{ padding:8px 12px; text-align:center; font-size:13px; }}
.heatmap th {{ background:#f0f0f0; font-weight:600; }}
.heatmap .pos {{ background:#bbf7d0; }}
.heatmap .neg {{ background:#fecaca; }}
.heatmap .neutral {{ background:#f9fafb; }}
</style>
</head>
<body>
<div class="container">
<h1>Strategy Evaluation Report</h1>
<p class="subtitle">Backtest {start} ~ {end} &middot; Initial Capital: {capital:.0} CNY</p>

<div class="card-grid">
  <div class="card"><div class="label">Total Return</div><div class="value positive">{total_ret:+.2}%</div></div>
  <div class="card"><div class="label">Annual Return</div><div class="value positive">{annual_ret:+.2}%</div></div>
  <div class="card"><div class="label">Max Drawdown</div><div class="value negative">{max_dd:.2}%</div></div>
  <div class="card"><div class="label">Sharpe Ratio</div><div class="value {sharpe_color}">{sharpe:.3}</div></div>
  <div class="card"><div class="label">Sortino Ratio</div><div class="value {sortino_color}">{sortino:.3}</div></div>
  <div class="card"><div class="label">Calmar Ratio</div><div class="value {calmar_color}">{calmar:.3}</div></div>
</div>

<div class="card-grid">
  <div class="card"><div class="label">Beta</div><div class="value">{beta:.3}</div></div>
  <div class="card"><div class="label">Alpha (annual)</div><div class="value {alpha_color}">{alpha:+.2}%</div></div>
  <div class="card"><div class="label">Risk-Free Rate</div><div class="value">{rf:.1}%</div></div>
</div>

<div class="chart-section">
  <h2>NAV Curve &mdash; Strategy vs Shanghai Composite</h2>
  <div class="chart-wrapper"><canvas id="navChart"></canvas></div>
</div>

<div class="chart-section">
  <h2>Drawdown</h2>
  <p style="color:#888;font-size:13px;margin-bottom:12px;">Max: {dd_pct:.2}% &middot; {dd_start} to {dd_end} &middot; {dd_days} days</p>
  <div class="chart-wrapper"><canvas id="ddChart"></canvas></div>
</div>

<div class="chart-section">
  <h2>Per-ETF Returns</h2>
  <div class="chart-wrapper"><canvas id="etfChart"></canvas></div>
</div>

<div class="chart-section">
  <h2>Rolling {win}-Day Risk Metrics</h2>
  <div class="chart-wrapper"><canvas id="rollingChart"></canvas></div>
</div>

<div class="chart-section">
  <h2>Weight Allocation</h2>
  <div class="chart-wrapper"><canvas id="weightChart"></canvas></div>
</div>

<div class="chart-section">
  <h2>Monthly Return Heatmap</h2>
  <div class="heatmap">{heatmap}</div>
</div>

<div class="chart-section">
  <h2>Dividend Timeline</h2>
  <div class="chart-wrapper"><canvas id="divChart"></canvas></div>
</div>

</div>

<script>
const NAV = {nav_json};
const BENCH = {bench_json};
const DD = {dd_json};
const ETF = {etf_json};
const ROLLING = {rolling_json};
const REBAL = {rebalance_json};
const DIV = {div_json};

Chart.defaults.font.family = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
Chart.defaults.plugins.tooltip.backgroundColor = "rgba(0,0,0,.75)";

new Chart(document.getElementById('navChart'), {{
  type:'line',
  data:{{
    labels:NAV.map(d=>d.date),
    datasets:[
      {{ label:'Strategy NAV', data:NAV.map(d=>d.val), borderColor:'#3b82f6', backgroundColor:'rgba(59,130,246,.1)', fill:true, tension:.3, pointRadius:0 }},
      {{ label:'SSE Composite (norm)', data:BENCH.map(d=>d.val), borderColor:'#94a3b8', borderDash:[5,3], tension:.3, pointRadius:0 }}
    ]
  }},
  options:{{
    responsive:true,
    plugins:{{ legend:{{ position:'top' }} }},
    scales:{{ x:{{ display:true }}, y:{{ title:{{ display:true, text:'Normalized Value' }} }} }}
  }}
}});

new Chart(document.getElementById('ddChart'), {{
  type:'line',
  data:{{
    labels:DD.map(d=>d.date),
    datasets:[{{
      label:'Drawdown %',
      data:DD.map(d=>d.val),
      borderColor:'#ef4444',
      backgroundColor:'rgba(239,68,68,.15)',
      fill:true,
      tension:.3,
      pointRadius:0
    }}]
  }},
  options:{{
    responsive:true,
    plugins:{{ legend:{{ display:false }} }},
    scales:{{
      x:{{ display:true }},
      y:{{ title:{{ display:true, text:'Drawdown (%)' }}, min:Math.min(-50, Math.min(...DD.map(d=>d.val))*1.1), max:0 }}
    }}
  }}
}});

new Chart(document.getElementById('etfChart'), {{
  type:'bar',
  data:{{
    labels:ETF.map(d=>d.label),
    datasets:[
      {{ label:'Total Return %', data:ETF.map(d=>d.ret), backgroundColor:ETF.map(d=>d.ret>=0?'#22c55e':'#ef4444'), indexAxis:'y' }}
    ]
  }},
  options:{{
    indexAxis:'y',
    responsive:true,
    plugins:{{ legend:{{ display:false }} }},
    scales:{{ x:{{ title:{{ display:true, text:'Return (%)' }} }} }}
  }}
}});

new Chart(document.getElementById('rollingChart'), {{
  type:'line',
  data:{{
    labels:ROLLING.map(d=>d.date),
    datasets:[
      {{ label:'Sharpe', data:ROLLING.map(d=>d.sharpe), borderColor:'#3b82f6', tension:.3, pointRadius:0, yAxisID:'y' }},
      {{ label:'Sortino', data:ROLLING.map(d=>d.sortino), borderColor:'#8b5cf6', tension:.3, pointRadius:0, yAxisID:'y' }},
      {{ label:'Beta', data:ROLLING.map(d=>d.beta), borderColor:'#f59e0b', borderDash:[3,2], tension:.3, pointRadius:0, yAxisID:'y1' }}
    ]
  }},
  options:{{
    responsive:true,
    plugins:{{ legend:{{ position:'top' }} }},
    scales:{{
      y:{{ type:'linear', position:'left', title:{{ display:true, text:'Ratio' }} }},
      y1:{{ type:'linear', position:'right', title:{{ display:true, text:'Beta' }}, grid:{{ drawOnChartArea:false }} }}
    }}
  }}
}});

new Chart(document.getElementById('weightChart'), {{
  type:'bar',
  data:{{
    labels:REBAL.labels,
    datasets:REBAL.datasets.map((ds,i)=>({{
      label:ds.label,
      data:ds.data,
      backgroundColor:['#3b82f6','#22c55e','#14b8a6','#f59e0b','#8b5cf6','#94a3b8'][i%6]
    }}))
  }},
  options:{{
    responsive:true,
    plugins:{{ legend:{{ position:'bottom' }} }},
    scales:{{
      x:{{ stacked:true }},
      y:{{ stacked:true, max:100, title:{{ display:true, text:'Weight (%)' }} }}
    }}
  }}
}});

new Chart(document.getElementById('divChart'), {{
  type:'bar',
  data:{{
    labels:DIV.labels,
    datasets:DIV.datasets
  }},
  options:{{
    responsive:true,
    plugins:{{ legend:{{ position:'bottom' }} }},
    scales:{{
      x:{{ title:{{ display:true, text:'Date' }} }},
      y:{{ title:{{ display:true, text:'Amount (CNY)' }} }}
    }}
  }}
}});
</script>
</body>
</html>"#,
        start = nav_curve.first().map(|n| n.date.as_str()).unwrap_or(""),
        end = nav_curve.last().map(|n| n.date.as_str()).unwrap_or(""),
        capital = 100_000.0,
        total_ret = metrics.total_return * 100.0,
        annual_ret = metrics.annualized_return * 100.0,
        max_dd = metrics.max_drawdown * 100.0,
        sharpe = metrics.sharpe,
        sortino = metrics.sortino,
        calmar = metrics.calmar,
        sharpe_color = if metrics.sharpe >= 1.0 { "positive" } else if metrics.sharpe >= 0.0 { "" } else { "negative" },
        sortino_color = if metrics.sortino >= 1.0 { "positive" } else if metrics.sortino >= 0.0 { "" } else { "negative" },
        calmar_color = if metrics.calmar >= 1.0 { "positive" } else if metrics.calmar >= 0.0 { "" } else { "negative" },
        beta = metrics.beta,
        alpha = metrics.alpha * 100.0,
        alpha_color = if metrics.alpha >= 0.0 { "positive" } else { "negative" },
        rf = metrics.risk_free_rate * 100.0,
        dd_pct = dd_info.max_drawdown * 100.0,
        dd_start = dd_info.start_date,
        dd_end = dd_info.end_date,
        dd_days = dd_info.duration_days,
        win = super::metrics::ROLLING_WINDOW,
        heatmap = heatmap_rows,
        nav_json = nav_json,
        bench_json = bench_json,
        dd_json = dd_json,
        etf_json = etf_json,
        rolling_json = rolling_json,
        rebalance_json = rebalance_json,
        div_json = div_json,
    )
}

pub struct MonthlyCell {
    pub year: i32,
    pub month: u32,
    pub value: f64,
}

fn heatmap_to_html(cells: &[MonthlyCell]) -> String {
    let mut rows = String::new();
    rows.push_str("<table><tr><th>Year</th>");
    for m in 1..=12 {
        rows.push_str(&format!("<th>{m:02}</th>"));
    }
    rows.push_str("</tr>");

    let years: Vec<i32> = {
        let mut ys: Vec<i32> = cells.iter().map(|c| c.year).collect();
        ys.sort();
        ys.dedup();
        ys
    };

    for y in years {
        rows.push_str(&format!("<tr><td><b>{y}</b></td>"));
        for m in 1..=12 {
            let val = cells.iter().find(|c| c.year == y && c.month == m).map(|c| c.value);
            match val {
                Some(v) => {
                    let cls = if v > 0.001 { "pos" } else if v < -0.001 { "neg" } else { "neutral" };
                    rows.push_str(&format!("<td class=\"{cls}\">{:+.1}%</td>", v * 100.0));
                }
                None => {
                    rows.push_str("<td class=\"neutral\">-</td>");
                }
            }
        }
        rows.push_str("</tr>");
    }
    rows.push_str("</table>");
    rows
}

fn nav_to_json(curve: &[NavPoint], _name: &str) -> String {
    let val = curve.first().map(|n| n.value).unwrap_or(1.0);
    let items: Vec<String> = curve
        .iter()
        .map(|n| format!(r#"{{"date":"{}","val":{:.4}}}"#, n.date, n.value / val))
        .collect();
    format!("[{}]", items.join(","))
}

fn dd_to_json(curve: &[NavPoint]) -> String {
    let peak = curve.first().map(|n| n.value).unwrap_or(1.0);
    let mut items = Vec::new();
    let mut running_peak = peak;
    for n in curve {
        if n.value > running_peak {
            running_peak = n.value;
        }
        let dd = (running_peak - n.value) / running_peak * -100.0;
        items.push(format!(r#"{{"date":"{}","val":{:.2}}}"#, n.date, dd));
    }
    format!("[{}]", items.join(","))
}

fn etf_to_json(returns: &HashMap<String, (f64, f64)>) -> String {
    let mut items: Vec<String> = returns
        .iter()
        .map(|(code, (ret, _dd))| {
            format!(r#"{{"label":"{}","ret":{:.2}}}"#, code, ret * 100.0)
        })
        .collect();
    items.sort();
    format!("[{}]", items.join(","))
}

fn rebalance_to_json(rebalance_events: &[RebalanceEvent], trade_events: &[TradeEvent]) -> String {
    let mut labels = Vec::new();
    let mut by_code: HashMap<String, Vec<f64>> = HashMap::new();
    let mut all_dates = Vec::new();

    // Initial allocation from trade events
    if !trade_events.is_empty() {
        let d = &trade_events[0].date;
        labels.push(format!("\"{}\"", d));
        all_dates.push(d.clone());
        for ev in trade_events {
            let w = ev.amount / 100_000.0 * 100.0;
            by_code.entry(ev.code.clone()).or_default().push(w);
        }
        by_code.entry("CASH".to_string()).or_default().push(10.0);
    }

    for ev in rebalance_events {
        labels.push(format!("\"{}\"", ev.date));
        all_dates.push(ev.date.clone());
        for (code, w) in &ev.weights_after {
            by_code.entry(code.clone()).or_default().push(w * 100.0);
        }
    }

    let ds: Vec<String> = by_code
        .iter()
        .map(|(code, vals)| {
            let data: Vec<String> = vals.iter().map(|v| format!("{v:.1}")).collect();
            format!(r#"{{"label":"{}","data":[{}]}}"#, code, data.join(","))
        })
        .collect();

    format!(r#"{{"labels":[{}],"datasets":[{}]}}"#, labels.join(","), ds.join(",\n"))
}

fn rolling_to_json(points: &[RollingPoint]) -> String {
    let items: Vec<String> = points
        .iter()
        .map(|p| {
            format!(
                r#"{{"date":"{}","sharpe":{:.3},"sortino":{:.3},"beta":{:.3}}}"#,
                p.date, p.sharpe, p.sortino, p.beta
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn div_to_json(events: &[DividendEvent]) -> String {
    let labels: Vec<String> = events.iter().map(|e| format!("\"{}\"", e.date)).collect();

    let mut by_code: HashMap<String, Vec<f64>> = HashMap::new();
    for ev in events {
        by_code.entry(ev.code.clone()).or_default().push(ev.total_income);
    }

    let all_codes: Vec<&String> = by_code.keys().collect();
    let colors = ["#3b82f6", "#22c55e", "#f59e0b", "#8b5cf6", "#ef4444"];

    let mut ds = Vec::new();
    for (i, code) in all_codes.iter().enumerate() {
        let vals: Vec<String> = (0..events.len()).map(|j| {
            if events[j].code == **code { format!("{:.2}", events[j].total_income) } else { "0".to_string() }
        }).collect();
        ds.push(format!(
            r#"{{"label":"{}","data":[{}],"backgroundColor":"{}"}}"#,
            code, vals.join(","), colors[i % colors.len()]
        ));
    }

    format!(r#"{{"labels":[{}],"datasets":[{}]}}"#, labels.join(","), ds.join(","))
}
