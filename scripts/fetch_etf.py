"""Download daily K-line data for specified ETFs via akshare."""

import akshare as ak
import pandas as pd

ETF_CODES = ["159941", "159581", "561580", "563020", "518880"]
START_DATE = "20250101"
OUTPUT_DIR = "./data"

def fetch_one(code: str) -> pd.DataFrame:
    df: pd.DataFrame = ak.fund_etf_hist_em(
        symbol=code,
        period="daily",
        start_date=START_DATE,
        end_date="20500101",
        adjust="qfq",
    )

    df = df.rename(columns={
        "日期": "date",
        "开盘": "open",
        "收盘": "close",
        "最高": "high",
        "最低": "low",
        "成交量": "volume",
        "成交额": "amount",
        "振幅": "amplitude",
        "涨跌幅": "pct_change",
        "涨跌额": "change",
        "换手率": "turnover",
    })

    df["date"] = pd.to_datetime(df["date"])
    return df


def main():
    for code in ETF_CODES:
        print(f"Fetching {code} ...")
        try:
            df = fetch_one(code)
            path = f"{OUTPUT_DIR}/{code}.csv"
            df.to_csv(path, index=False, encoding="utf-8-sig")
            print(f"  -> {path}  ({len(df)} rows, {df['date'].iloc[0].date()} ~ {df['date'].iloc[-1].date()})")
        except Exception as e:
            print(f"  -> FAILED: {e}")


if __name__ == "__main__":
    main()
