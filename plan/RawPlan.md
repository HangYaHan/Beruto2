下面直接给你一套**可执行的调仓伪代码**。  
我会按你现在的目标配置来写，并尽量保持“策略 A：保守型”的原则：**低频、规则少、先控回撤，再谈收益**。

---

# 1. 目标配置

## 1.1 战略目标权重

- 纳指总仓：$40\%$
  - 纳指底仓：$25\%$
  - 纳指卫星仓：$15\%$
- A 股红利总仓：$40\%$
  - 红利核心仓：$30\%$
  - 红利战术仓：$10\%$
- 黄金：$10\%$
- 现金/短债：$10\%$

记为：

$$
w_{\text{NDX,core}} = 25\%
$$

$$
w_{\text{NDX,sat}} = 15\%
$$

$$
w_{\text{DIV,core}} = 30\%
$$

$$
w_{\text{DIV,tac}} = 10\%
$$

$$
w_{\text{Gold,target}} = 10\%
$$

$$
w_{\text{Cash,target}} = 10\%
$$

---

# 2. 变量定义

为了让规则明确，先定义变量。

## 2.1 纳指相关变量

- $P_{\text{NDX}}$：纳指 ETF 当前价格
- $MA_{10m,\text{NDX}}$：纳指 $10$ 月均线
- $DD_{\text{NDX}}$：纳指从最近高点的回撤幅度
- $VIX$：波动率指数
- $EPSRev_{\text{NDX}}$：纳指或美股科技板块未来 $12$ 个月盈利预期修正指标
- $Breadth_{\text{NDX}}$：纳指市场广度指标
- $PEPct_{\text{NDX}}$：纳指估值分位数

## 2.2 A 股红利相关变量

- $P_{\text{DIV}}$：A 股红利 ETF 或指数价格
- $MA_{10m,\text{DIV}}$：A 股红利 $10$ 月均线
- $DD_{\text{DIV}}$：A 股红利回撤幅度
- $ValPct_{\text{DIV}}$：A 股红利估值分位数
- $DY_{\text{DIV}}$：A 股红利股息率
- $Crowd_{\text{DIV}}$：拥挤度指标

## 2.3 黄金相关变量

- $w_{\text{Gold}}$：黄金当前权重

## 2.4 现金相关变量

- $w_{\text{Cash}}$：现金/短债当前权重

---

# 3. 总体执行原则

先把最重要的约束写清楚。

---

## 3.1 原则一：核心仓低频，不做主观择时

```text
NDX_core: 不因宏观叙事主动卖出，只允许定投、再平衡、极端风险控制
DIV_core: 原则上长期持有，只做季度/半年再平衡
Gold: 只做低频再平衡
Cash: 作为缓冲和战术调仓资金来源
```

---

## 3.2 原则二：战术仓只按规则动，不按感觉动

```text
NDX_sat: 由趋势、回撤、波动、EPS 修正驱动
DIV_tac: 由估值、拥挤度、趋势驱动，但动作比纳指更轻
```

---

## 3.3 原则三：调仓频率分层

```text
每月末：处理纳指卫星仓、定投、检查趋势
每季度末：处理红利战术仓、黄金、总仓再平衡
极端行情：若满足预设条件，可临时执行一次应急调仓
```

---

# 4. 纳指调仓伪代码

这是整套系统最核心的一部分。

---

## 4.1 纳指底仓规则

### 逻辑
纳指底仓用于长期持有，不因为“担心高位”随意卖出。

```text
IF rebalance_date == TRUE THEN
    rebalance NDX_core toward 25%
END IF

IF monthly_DCA_date == TRUE THEN
    add new contribution to NDX_core
END IF

# 禁止以下行为
DO NOT sell NDX_core because of:
    - AI bubble narrative
    - election narrative
    - USD system narrative
    - "price too high" feeling
```

---

## 4.2 纳指卫星仓：减仓规则

### 一级减仓
当趋势开始变弱时，先减一半卫星仓。

```text
IF month_end(P_NDX < MA_10m_NDX) FOR 2 consecutive months THEN
    target_w_NDX_sat = 7.5%
END IF
```

解释：  
原本纳指卫星仓目标为 $15\%$，若月线跌破长期均线并连续两个月未收复，说明中期趋势恶化，先减半。

---

### 二级减仓
若趋势恶化叠加风险放大，再进一步降低卫星仓。

```text
IF month_end(P_NDX < MA_10m_NDX) FOR 2 consecutive months
   AND DD_NDX > 15%
   AND VIX > 30
   AND EPSRev_NDX < 0 THEN
       target_w_NDX_sat = 0% to 5%
END IF
```

解释：  
这代表不是普通回调，而更像进入系统性风险环境。此时保守型策略应优先减少回撤。

---

## 4.3 纳指卫星仓：恢复规则

### 恢复到一半
```text
IF month_end(P_NDX > MA_10m_NDX)
   AND VIX is falling
   AND EPSRev_NDX stabilizing THEN
       target_w_NDX_sat = 7.5%
END IF
```

### 恢复到满仓
```text
IF month_end(P_NDX > MA_10m_NDX) FOR 2 consecutive months
   AND Breadth_NDX improving
   AND EPSRev_NDX >= 0 THEN
       target_w_NDX_sat = 15%
END IF
```

解释：  
你不能在刚反弹一下就全恢复。  
保守策略要接受“慢一点”，换取更少误判。

---

## 4.4 纳指卫星仓：高估值暂停加仓规则

```text
IF PEPct_NDX > 85%
   AND P_NDX > 1.12 * MA_10m_NDX
   AND EPSRev_NDX weakening THEN
       pause_new_buying_NDX_sat = TRUE
ELSE
       pause_new_buying_NDX_sat = FALSE
END IF
```

解释：  
这不是卖出信号，而是**暂停新增卫星仓追高**。  
核心仓照常，卫星仓不加速。

---

## 4.5 纳指卫星仓：回撤加仓规则

```text
IF DD_NDX >= 15% AND w_Cash > 7.5% THEN
    buy 2.5% portfolio into NDX_sat
    reduce Cash by 2.5%
END IF

IF DD_NDX >= 20% AND VIX > 25 AND w_Cash > 5% THEN
    buy another 2.5% portfolio into NDX_sat
    reduce Cash by 2.5%
END IF

IF DD_NDX >= 25% AND EPSRev_NDX not collapsing completely AND w_Cash > 2.5% THEN
    buy another 2.5% portfolio into NDX_sat
    reduce Cash by 2.5%
END IF
```

解释：  
这部分是保守策略里少数“主动出击”的部分，但你不能一次用光现金。  
要留最后一部分缓冲。

---

# 5. A 股红利调仓伪代码

A 股红利不建议像纳指那样做过强趋势交易。  
它更适合轻战术化。

---

## 5.1 红利核心仓规则

```text
IF quarterly_rebalance_date == TRUE THEN
    rebalance DIV_core toward 30%
END IF

IF monthly_DCA_date == TRUE THEN
    add new contribution to DIV_core
END IF
```

---

## 5.2 红利战术仓：暂停新增规则

```text
IF ValPct_DIV > 85%
   AND DY_DIV falling significantly
   AND Crowd_DIV high THEN
       pause_new_buying_DIV_tac = TRUE
ELSE
       pause_new_buying_DIV_tac = FALSE
END IF
```

解释：  
高股息资产也可能被交易拥挤。  
这时先暂停新增，不必急着砍核心仓。

---

## 5.3 红利战术仓：轻度减仓规则

如果你希望保守一点，可以加入一个很轻的战术规则。

```text
IF month_end(P_DIV < MA_10m_DIV)
   AND ValPct_DIV > 80%
   AND Crowd_DIV high THEN
       target_w_DIV_tac = 5%
END IF
```

即从 $10\%$ 战术仓降到 $5\%$。

---

## 5.4 红利战术仓：恢复规则

```text
IF month_end(P_DIV > MA_10m_DIV)
   AND DY_DIV stabilizing
   AND Crowd_DIV normalizing THEN
       target_w_DIV_tac = 10%
END IF
```

解释：  
红利战术仓的变化要轻，不要像交易成长股那样激烈。

---

## 5.5 红利战术仓：回撤补仓规则

```text
IF DD_DIV >= 12%
   AND ValPct_DIV <= historical_median
   AND DY_DIV rising THEN
       add 2.5% portfolio into DIV_tac from Cash
END IF
```

但注意：  
如果你已经把现金优先留给纳指回撤，那这里要有优先级规则。

---

# 6. 黄金调仓伪代码

黄金在这个组合里不是战术交易资产，而是结构性对冲资产。

---

## 6.1 黄金只做区间再平衡

```text
IF quarterly_rebalance_date == TRUE THEN
    IF w_Gold > 12% THEN
        sell Gold down to 10%
        move excess to Cash
    END IF

    IF w_Gold < 8% THEN
        buy Gold up to 10% using Cash
    END IF
END IF
```

解释：  
黄金在 $8\%$ 到 $12\%$ 之间自然波动即可。  
你不需要对它做复杂择时。

---

# 7. 现金/短债调仓伪代码

现金仓是你的缓冲器，不是“闲置资产”。

---

## 7.1 现金下限规则

```text
minimum_cash_floor = 2.5%
preferred_cash_floor = 5%
```

### 原则
- 尽量不要把现金仓打到 $2.5\%$ 以下；
- 正常情况下尽量维持至少 $5\%$ 现金/短债。

---

## 7.2 现金优先级规则

如果纳指和红利同时回撤，需要明确优先顺序。

```text
IF both NDX and DIV trigger buy signals THEN
    prioritize NDX_sat first
    allocate remaining cash to DIV_tac only if Cash > preferred_cash_floor
END IF
```

解释：  
因为纳指波动更大、回撤更深、预期收益弹性也更高，所以现金优先给纳指卫星仓更合理。

---

# 8. 组合总再平衡伪代码

这是防止权重飘移的总控层。

---

## 8.1 季度检查规则

```text
FOR each asset_bucket in [NDX_core, NDX_sat, DIV_core, DIV_tac, Gold, Cash]:
    IF current_weight > 1.2 * target_weight OR current_weight < 0.8 * target_weight THEN
        mark_for_rebalance = TRUE
    END IF
END FOR
```

---

## 8.2 半年强制再平衡规则

```text
IF semiannual_rebalance_date == TRUE THEN
    rebalance all asset buckets toward strategic targets
    subject to current tactical overrides
END IF
```

这里的“subject to current tactical overrides”意思是：

- 如果纳指卫星仓当前因为风控规则只能是 $7.5\%$；
- 那么半年再平衡时，也不要强行把它拉回 $15\%$；
- 应尊重战术层当前状态。

---

# 9. 完整版总伪代码

下面给你一个可以直接抄进文档的整合版。

```text
# =========================
# TARGET WEIGHTS
# =========================
NDX_core_target = 25%
NDX_sat_target_full = 15%
DIV_core_target = 30%
DIV_tac_target_full = 10%
Gold_target = 10%
Cash_target = 10%

NDX_sat_target_current = 15%
DIV_tac_target_current = 10%

minimum_cash_floor = 2.5%
preferred_cash_floor = 5%

# =========================
# MONTHLY PROCESS
# =========================

# 1. Monthly DCA
IF monthly_DCA_date == TRUE THEN
    buy NDX_core
    buy DIV_core
END IF

# 2. Nasdaq satellite risk reduction
IF month_end(P_NDX < MA_10m_NDX) FOR 2 consecutive months THEN
    NDX_sat_target_current = 7.5%
END IF

IF month_end(P_NDX < MA_10m_NDX) FOR 2 consecutive months
   AND DD_NDX > 15%
   AND VIX > 30
   AND EPSRev_NDX < 0 THEN
    NDX_sat_target_current = 0% to 5%
END IF

# 3. Nasdaq satellite recovery
IF month_end(P_NDX > MA_10m_NDX)
   AND VIX falling
   AND EPSRev_NDX stabilizing THEN
    NDX_sat_target_current = max(NDX_sat_target_current, 7.5%)
END IF

IF month_end(P_NDX > MA_10m_NDX) FOR 2 consecutive months
   AND Breadth_NDX improving
   AND EPSRev_NDX >= 0 THEN
    NDX_sat_target_current = 15%
END IF

# 4. Nasdaq pause new buying
IF PEPct_NDX > 85%
   AND P_NDX > 1.12 * MA_10m_NDX
   AND EPSRev_NDX weakening THEN
    pause_new_buying_NDX_sat = TRUE
ELSE
    pause_new_buying_NDX_sat = FALSE
END IF

# 5. Nasdaq opportunistic buying from cash
IF DD_NDX >= 15% AND w_Cash > 7.5% THEN
    buy 2.5% into NDX_sat
    w_Cash = w_Cash - 2.5%
END IF

IF DD_NDX >= 20% AND VIX > 25 AND w_Cash > 5% THEN
    buy another 2.5% into NDX_sat
    w_Cash = w_Cash - 2.5%
END IF

IF DD_NDX >= 25% AND EPSRev_NDX not collapsing completely AND w_Cash > minimum_cash_floor THEN
    buy another 2.5% into NDX_sat
    w_Cash = w_Cash - 2.5%
END IF

# =========================
# QUARTERLY PROCESS
# =========================

# 6. Dividend tactical pause
IF ValPct_DIV > 85%
   AND DY_DIV falling significantly
   AND Crowd_DIV high THEN
    pause_new_buying_DIV_tac = TRUE
ELSE
    pause_new_buying_DIV_tac = FALSE
END IF

# 7. Dividend tactical mild reduction
IF month_end(P_DIV < MA_10m_DIV)
   AND ValPct_DIV > 80%
   AND Crowd_DIV high THEN
    DIV_tac_target_current = 5%
END IF

# 8. Dividend tactical recovery
IF month_end(P_DIV > MA_10m_DIV)
   AND DY_DIV stabilizing
   AND Crowd_DIV normalizing THEN
    DIV_tac_target_current = 10%
END IF

# 9. Dividend opportunistic buying
IF DD_DIV >= 12%
   AND ValPct_DIV <= historical_median
   AND DY_DIV rising
   AND w_Cash > preferred_cash_floor THEN
    buy 2.5% into DIV_tac
    w_Cash = w_Cash - 2.5%
END IF

# 10. Gold rebalance
IF w_Gold > 12% THEN
    sell Gold down to 10%
    move excess to Cash
END IF

IF w_Gold < 8% THEN
    buy Gold up to 10% using Cash
END IF

# 11. Strategic rebalance check
FOR asset_bucket IN [NDX_core, NDX_sat, DIV_core, DIV_tac, Gold, Cash]:
    IF current_weight(asset_bucket) > 1.2 * target_weight(asset_bucket)
       OR current_weight(asset_bucket) < 0.8 * target_weight(asset_bucket) THEN
       rebalance_flag = TRUE
    END IF
END FOR

IF rebalance_flag == TRUE THEN
    rebalance toward target weights
    subject to:
        NDX_sat_target_current
        DIV_tac_target_current
        cash floor rules
END IF

# =========================
# SEMIANNUAL PROCESS
# =========================

IF semiannual_rebalance_date == TRUE THEN
    rebalance NDX_core toward 25%
    rebalance DIV_core toward 30%
    rebalance Gold toward 10%
    rebalance Cash toward 10%
    rebalance NDX_sat toward NDX_sat_target_current
    rebalance DIV_tac toward DIV_tac_target_current
END IF
```

---

# 10. 我帮你再指出几个执行时最容易犯的错误

---

## 10.1 错误一：把“接近触发”当“已经触发”

例如：

- 还没连续两个月跌破 $10$ 月均线；
- 你就提前减仓。

这会破坏系统。  
你必须接受：**规则慢一点，但更不容易被噪音骗。**

---

## 10.2 错误二：宏观叙事凌驾于规则之上

例如：

- 条件没触发，但你看到“特朗普”“AI 泡沫”“去美元化”就先动手。

那你等于嘴上说程序化，实际还是主观交易。

---

## 10.3 错误三：战术仓和核心仓混用

一旦你开始动核心仓，这套策略就变形了。

- 核心仓负责长期信念；
- 战术仓负责风险调节。

不能混。

---

## 10.4 错误四：组合再平衡过于频繁

如果你每个月都小修小补，交易频率会上去，系统也会失真。  
策略 A 的本质是：**慢、少、硬规则。**

---

# 11. 最后给你一个更短的“口袋版伪代码”

如果你只想记住核心几条，就记这个：

```text
# NDX
IF NDX monthly close < MA_{10m} for 2 months:
    cut NDX_sat from 15% to 7.5%

IF above condition + DD > 15% + VIX > 30 + EPSRev < 0:
    cut NDX_sat to 0%~5%

IF NDX monthly close > MA_{10m} and stabilizing:
    restore NDX_sat to 7.5%

IF NDX monthly close > MA_{10m} for 2 months + Breadth improves:
    restore NDX_sat to 15%

IF NDX DD >= 15%:
    use 2.5% Cash to buy NDX_sat
IF NDX DD >= 20%:
    use another 2.5% Cash
IF NDX DD >= 25%:
    use another 2.5% Cash

# DIV
IF DIV valuation high + dividend yield falling + crowding high:
    stop adding DIV_tac

IF DIV monthly close < MA_{10m} and crowding high:
    cut DIV_tac from 10% to 5%

IF DIV recovers above MA_{10m} and crowding normalizes:
    restore DIV_tac to 10%

# Gold
Keep Gold in 8%~12% range

# Rebalance
Quarterly check; semiannual forced rebalance
Do not break cash floor
Do not touch core positions except rebalance
```

---

如果你愿意，我下一步可以继续给你两种更落地的版本之一：

1. **把这套伪代码翻成 Excel 列字段 + 公式判断表**；  
2. **把这套策略写成真正像程序一样的 Python 风格伪代码 / 状态机版本**。