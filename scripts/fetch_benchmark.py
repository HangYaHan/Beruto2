"""Fetch Shanghai Composite Index daily data for benchmark."""

import akshare as ak
import pandas as pd

START_DATE = "20250101"
OUTPUT = "./data/benchmark.csv"


def main():
    print("Fetching Shanghai Composite Index (000001) ...")
    df: pd.DataFrame = ak.stock_zh_index_daily(symbol="sh000001")

    df = df.rename(columns={
        "date": "date",
        "close": "close",
    })

    df["date"] = pd.to_datetime(df["date"])
    df = df[df["date"] >= START_DATE]
    df = df[["date", "close"]].sort_values("date").reset_index(drop=True)
    df.to_csv(OUTPUT, index=False, encoding="utf-8-sig")
    print(f"Saved {len(df)} rows to {OUTPUT}")
    print(f"  {df['date'].iloc[0].date()} ~ {df['date'].iloc[-1].date()}")


if __name__ == "__main__":
    main()
