# 可复用工程实践 & 提示词策略

从 Beruto ETF 回测工具链开发过程中提炼出的模式和技巧。

---

## 1. 项目元文件：AGENTS.md

### 做法

在项目启动时（或任何修改之前）创建 `AGENTS.md`，作为 AI Agent 与项目的约定文件，包含以下内容：

```markdown
## Project        — 一句话描述项目
## Agent Instructions — 行为准则（think step by step, UNIX Philosophy, 语言偏好）
## Data Rules     — 数据读写权限、字段映射、输出路径
## Structure      — 目录/文件 → 用途 的映射表
## Commands       — 所有可用命令（fetch / build / test / run）
## Dependencies   — 工具链说明
## Quirks         — 已知坑点（字体、版本兼容等）
```

### 价值

- 消除 Agent 的猜测：哪里能写、哪里不能写、怎么运行、怎么测试一目了然
- 减少重复提问：符号映射、费用率等全局常量一次性定义
- 新 Agent 对话可立即获得完整上下文
- 人类和 Agent 都能通过一个文件理解项目的"玩法"

### 关键设计点

| 条目 | 说明 |
|------|------|
| **Data Rules** | `data/` 只读，除非用户明确要求才可写入。策略实现时只能用已有 CSV，缺失则中止 |
| **Symbol mapping** | `159941`=纳指, `518880`=黄金, `159581/561580/563020`=红利（等分） |
| **Output path** | 所有产出物强制输出到 `result/` |
| **Fee rule** | ETF 交易费 0.025%，全局常量，买/卖都扣 |
| **Quirks** | 记录项目特有的坑，避免反复踩（如 mplfinance 无稳定版、CJK 字体静默失败） |

---

## 2. UNIX Philosophy 严格落地

### 原则

> 每个模块只做一件事，把它做好。

### 实践

```
src/
  data/mod.rs         — 只做 CSV 加载 + PriceData 定义，不管业务
  backtest/
    types.rs          — 纯数据结构，零逻辑
    portfolio.rs      — 持仓、现金、NAV、调仓（买/卖含手续费）
    strategy.rs       — Strategy trait + 具体策略实现（只管"要不要调、调成什么"）
    engine.rs         — 主循环（每天的 step、分红、调仓），不涉及指标计算
  evaluation/
    metrics.rs        — 纯数学函数，不依赖 HTML 或 IO
    benchmarks.rs     — 基准数据加载 + 日期对齐
    report.rs         — 纯 HTML 生成，不计算指标
    mod.rs            — 编排器：指标计算 → HTML 生成 → 写文件
```

### 价值

- 最小化单文件改动范围：修 Bug 只动一个文件
- 编译速度快（每个模块独立编译单元）
- 新策略只需实现 `Strategy` trait，其余全部复用
- 测试可以针对单一模块进行

---

## 3. 数据管线的三段式架构

### 模式

```
[Python 数据获取] → [Rust 回测引擎] → [Rust 评估 + HTML 输出]
  scripts/            src/backtest/       src/evaluation/
```

### 关键决策

| 决策 | 理由 |
|------|------|
| 数据获取用 Python | akshare 无 Rust 替代 |
| 回测/评估用 Rust | 性能 + 类型安全 |
| HTML 报告用 Rust 生成 | 单次 `cargo run` 完成全流程，无需额外 Node/Python  |
| Chart.js CDN 引用 | 零依赖，浏览器直接打开即可交互 |

### 管线原则

1. 每个阶段输出明确的中间产物（CSV → JSON → HTML）
2. 上游不确定时不做下游（数据不够就 abort，不强行运行）
3. 中间产物可独立检查（直接读 JSON 验证回测正确性，再生成 HTML）

---

## 4. TODO 驱动的分步执行

### 做法

复杂任务拆成小步骤，用 TODO 列表跟踪：

```
1. 创建 fetch_benchmark.py        (high)
2. 实现 benchmarks.rs             (high)
3. 实现 metrics.rs                (high)
4. 实现 report.rs                 (high)
5. 实现 mod.rs                    (high)
6. 更新 main.rs                   (high)
7. 构建、运行、验证               (medium)
```

### 原则

- **每次只在一个 in_progress**：完成一个再开始下一个
- **完成后立即标记 completed**：不攒到批量标记
- **按优先级排列**：high 优先，medium 排后
- **最后一项目是验证**：确保所有修改通过 build + run

### 价值

- 用户可随时了解进度
- Agent 不会被复杂任务淹没
- 中断后恢复时可立即知道做到哪了

---

## 5. Question Tool：模糊点的决策机制

### 做法

当遇到模糊需求时，使用 `question` 工具让用户选择，而不是猜测：

- **回测输出格式**：每个策略一个 JSON vs 统一格式 → 用户选统一格式
- **交易费率**：建议 0.025% → 用户确认
- **是否加入分红**：建议加入 → 用户确认
- **是否需要基准指数**：建议上证综指 → 用户确认
- **HTML 加 JS 还是纯静态截图**：建议 Chart.js 交互 → 用户确认
- **指标计算：Rust vs Python**：建议 Rust → 用户确认

### 原则

- 推荐选项标 `(Recommended)`
- 列出简洁的利弊说明
- 一次只问一组相关问题
- 决策结果写入 AGENTS.md，后续不再重复问

---

## 6. Git 安全意识

### 原则

- **永不主动 commit**：除非用户明确要求
- **永不修改 git config**
- **永不 force push**
- **Commit 前必须确认**：`git status` + `git diff` + `git log` 三板斧
- Commit 失败时 **不要 amend**，修复后重新 commit
- 提 PR 前检查历史提交，draft 简洁描述（1-3 bullet points）

---

## 7. Bug 定位的方法论

### 案例：交易手续费引入的 -91% 回测收益

**现象**：回测输出显示 -91% return

**排查步骤**：
1. 对比 Python 与 Rust 输出：Python 正常，Rust 异常 → 定位到 Rust 代码
2. 搜关键逻辑 `initial_allocate`：发现手续费计算行
3. 分析代码：`self.cash -= cost + fee` 在已从总资本分配持仓后额外扣了现金
4. 修正：移除重复扣款，手续费体现为份额减少而非现金减少

### 可复用模式

- **回归对比**：新旧实现对照运行，差异定位
- **缩小范围**：从输出向输入逐层排查
- **搜关键字**：定位到具体函数和行
- **理解意图再修代码**：不盲改，先理解正确逻辑

---

## 8. 增量构建 + 持续验证

### 做法

每个 Rust 文件写完立即：

```bash
cargo build    # 编译检查
```

模块完成后的完整流程：

```bash
cargo build    # 确认编译通过
cargo test     # 确认测试通过
cargo run      # 跑完整管线
```

### 价值

- 问题早暴露：不会 5 个文件都写完才发现拼写错误
- 减小调试范围：每次只改一个文件，出错就是它
- 最终 HTML 也用脚本验证：检查 chart IDs、表格、数据注入是否完整

---

## 9. JSON Schema 约定

### 做法

输出格式在 `types.rs` 中定义，用 `serde` 标注：

```rust
#[derive(Serialize, Deserialize)]
struct BacktestOutput {
    parameters: Parameters,      // 不变元信息
    nav_curve: Vec<NavPoint>,    // 时间序列
    events: Vec<StrategyEvent>,  // 交易/分红/调仓事件
    summary: StrategySummary,    // 汇总统计
}
```

### 价值

- 序列化/反序列化自动完成，无需手动 JSON 拼接
- 结构清晰，下游消费者（evaluation）能精确知道字段
- 扩展时只需加字段，不影响已有字段

---

## 10. HTML 报告的单文件策略

### 做法

- 纯 HTML 单文件，含内嵌 CSS + Chart.js CDN 引用
- 所有数据以 JSON 格式 inline 注入 `<script>` 标签
- 无需本地服务器、无需 webpack、无需 npm

### 优势

- 输出即用：浏览器直接打开 `result/report.html`
- 零运行时依赖（除 CDN 的 Chart.js）
- Rust `format!()` 生成即可，无需引入模板引擎

---

## 11. 并行工具调用

### 做法

独立操作同时发出，不串行等待：

```
并行调用：
  - cargo build
  - cargo test
  - read Cargo.toml
  - read src/main.rs
```

### 原则

- 无依赖关系的操作用并行
- 有依赖的串行（如 git add → git commit → git status）
- shell 中避免 `&&`（PowerShell 不兼容），用 `; `

---

## 12. 文档随项目演进

### 做法

- 新建文件/模块后同步更新 AGENTS.md 的 Structure 表
- 新增命令后同步更新 Commands 区块
- 项目完成后用 README.md 给人类看，AGENTS.md 给 Agent 看

### 两者分工

| 文件 | 受众 | 内容侧重 |
|------|------|---------|
| `README.md` | 人类开发者 | 概述、快速开始、数据资产、最终指标 |
| `AGENTS.md` | AI Agent | 规则、约束、映射、命令、坑点 |

---

## 13. 提示词技巧总结

| 技巧 | 示例 |
|------|------|
| **思维链条强制** | "Think step by step before taking any action" |
| **语言偏好明确** | "Prefer Rust for all new code" — 避免 Agent 用 Python 写核心逻辑 |
| **输出约束** | "Only output to `./result/`" — 避免污染源码目录 |
| **中止条件** | "If symbol has no data file, abort and ask" — 防止盲目拉数据 |
| **结束语模板** | "output a brief summary in Chinese (中文)" — 用户母语反馈 |
| **格式决策前询问** | 用 question tool 提供选项 + 推荐 — 减少返工 |

---

## 14. 工具选择矩阵

| 场景 | 工具 | 原因 |
|------|------|------|
| 搜索文件 | Glob | 比 `find` 更快 |
| 搜索代码内容 | Grep | 支持正则，比 `rg` 更精确输出 |
| 读取文件 | Read | 比 `cat`/`head` 更好（line number） |
| 编辑文件 | Edit | 精确替换，比 `sed` 更安全 |
| 写新文件 | Write | 比 `echo >` 更可靠 |
| 终端命令 | Bash | 构建、测试、git 操作 |
| 复杂分步任务 | Task + subagent | 并行工作 |
| 模糊决策 | Question | 让用户选，不猜测 |

---

## 总结

本次对话产出了一个完整的数据管线项目，核心可迁移的经验是：

1. **AGENTS.md 作为项目契约**是最高杠杆的实践 — 花 5 分钟写好，省下无数轮纠正
2. **UNIX Philosophy 分模块**让每个文件 <200 行且职责单一，Bug 定位秒级
3. **TODO + 增量验证**确保每一步都正确，绝不积压问题
4. **HTML 单文件 + CDN** 是最适合 Rust 后端的报告方案，零额外复杂度
5. **question tool 在模糊点及时决策**，避免 Agent 猜测导致返工
