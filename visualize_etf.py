"""Visualize ETF K-line chart with daily/weekly/monthly scales via mplfinance."""

import argparse
import matplotlib
import pandas as pd
import mplfinance as mpf

RESAMPLE_RULES = {
    "w": {"rule": "W-FRI", "label": "Weekly K"},
    "m": {"rule": "ME", "label": "Monthly K"},
}


def load_data(code: str) -> pd.DataFrame:
    df = pd.read_csv(f"./data/{code}.csv", parse_dates=["date"])
    df = df.set_index("date")
    return df[["open", "high", "low", "close", "volume"]]


def resample_ohlcv(df: pd.DataFrame, rule: str) -> pd.DataFrame:
    return df.resample(rule).agg({
        "open": "first",
        "high": "max",
        "low": "min",
        "close": "last",
        "volume": "sum",
    }).dropna()


def plot_kline(df: pd.DataFrame, code: str, title_suffix: str, save: str | None = None):
    mpf.plot(
        df,
        type="candle",
        volume=True,
        title=f"{code} — {title_suffix}",
        style="charles",
        figsize=(16, 8),
        ylabel="Price",
        ylabel_lower="Volume",
        datetime_format="%Y-%m-%d",
        xrotation=30,
        savefig=save,
    )


def main():
    parser = argparse.ArgumentParser(description="Plot ETF K-line chart")
    parser.add_argument("code", nargs="?", default="159941",
                        help="ETF code in ./data/ (default: 159941)")
    parser.add_argument("--period", "-p", choices=["d", "w", "m"], default="d",
                        help="Time scale: d=daily, w=weekly, m=monthly (default: d)")
    parser.add_argument("--save", "-s", nargs="?", const="auto", default=None,
                        help="Save to PNG instead of showing (optional path)")
    args = parser.parse_args()

    df = load_data(args.code)

    if args.period == "d":
        label = "Daily K"
    else:
        cfg = RESAMPLE_RULES[args.period]
        df = resample_ohlcv(df, cfg["rule"])
        df.index = df.index.to_period(args.period.upper()).to_timestamp()
        label = cfg["label"]

    save_path = None
    if args.save is not None:
        save_path = args.save if args.save != "auto" else f"./result/{args.code}_{args.period}.png"
        matplotlib.use("Agg")

    plot_kline(df, args.code, label, save=save_path)

    if save_path:
        print(f"Saved: {save_path}")
    elif "Agg" not in matplotlib.get_backend():
        import matplotlib.pyplot as plt
        plt.show()


if __name__ == "__main__":
    main()
