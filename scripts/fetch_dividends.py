"""Fetch and cache dividend data for dividend ETFs."""

import akshare as ak
import pandas as pd


DIV_CODES = ["159581", "561580", "563020"]
OUTPUT = "./data/etf_dividends.csv"


def fetch_dividends(code: str) -> pd.DataFrame:
    rows = []
    for year in ["2025", "2026"]:
        df = ak.fund_fh_em(year=year, page=-1)
        cols = df.columns.tolist()
        code_col = cols[1]
        exdate_col = cols[4]
        div_col = cols[5]
        mask = df[code_col] == code
        matched = df.loc[mask, [exdate_col, div_col]].copy()
        matched.columns = ["ex_date", "amount_per_share"]
        matched["code"] = code
        rows.append(matched)
    return pd.concat(rows, ignore_index=True)


def main():
    all_rows = []
    for code in DIV_CODES:
        print(f"Fetching dividends for {code} ...")
        df = fetch_dividends(code)
        df["ex_date"] = pd.to_datetime(df["ex_date"]).dt.strftime("%Y-%m-%d")
        df["amount_per_share"] = df["amount_per_share"].astype(float)
        all_rows.append(df)
        print(f"  -> {len(df)} records")

    result = pd.concat(all_rows, ignore_index=True)
    result = result.sort_values(["code", "ex_date"]).reset_index(drop=True)
    result.to_csv(OUTPUT, index=False, encoding="utf-8-sig")
    print(f"\nSaved {len(result)} dividend records to {OUTPUT}")


if __name__ == "__main__":
    main()
